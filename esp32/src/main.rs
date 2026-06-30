use enumset::enum_set;
use esp_idf_svc::{
    eventloop::{EspEventLoop, EspSystemEventLoop, System},
    hal::{
        cpu::Core, gpio::AnyOutputPin, peripheral::Peripheral, peripherals::Peripherals,
        task::watchdog::TWDTConfig,
    },
    log::EspLogger,
    nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault},
    sys::link_patches,
};
use std::{thread::sleep, time::Duration};

use crate::{
    accu::{single_nimh_cell_volt_to_percent, Accu},
    pumps::{PumpError, Pumps},
    query::fetch_jobs,
    status_signaler::StatusSignaler,
    wifi_connect::connect_to_wifi_with_timeout,
};

mod accu;
mod deepsleep;
mod pumps;
mod query;
mod status_signaler;
mod wifi_connect;

// Binary on three LEDs
const SIGNAL_WHILE_WIFI: u8 = 1;
const SIGNAL_WHILE_FETCH: u8 = 2;
const SIGNAL_WHILE_WATERING: u8 = 4;
const ERROR_SLEEP_DURATION: Duration = Duration::from_secs(7200);
const ERROR_SHOW_RED_LED_DURATION: Duration = Duration::from_secs(15);

const ACCU_NIMH_CELLS_IN_ROW: usize = 8;
const ACCU_VOLTAGE_R1: f32 = 98.3; // kOhm
const ACCU_VOLTAGE_R2: f32 = 32.6; // kOhm
const ACCU_VOLTAGE_FACTOR: f32 = (ACCU_VOLTAGE_R1 + ACCU_VOLTAGE_R2) / ACCU_VOLTAGE_R2;
const ACCU_CRITICAL_VOLTAGE: f32 = 4.0; // stop watering below critical volt (under load)

// Measure how much the pump outputs continuously over 10s.
// Measure the result and divide by 10s.
// Measure the voltage of the battery while pumping.
// Divide the previous value by volt.
// I measured 1300ml / 10s / 5.8V
pub const PUMP_ML_PER_VOLT_SECOND: f32 = 22.413793;
pub const PUMP_WARMUP_MS: u32 = 25; // no abrupt start
pub const PUMP_COOLDOWN_MS: u32 = 25; // no abrupt stop

// Hard coded maximum, which will never be exceeded.
pub const PUMP_MAX_PUMP_DURATION: Duration = Duration::from_secs(15);

#[derive(Debug)]
enum RoutineError {
    Watchdog,
    Wifi(wifi_connect::WifiErr),
    Query(query::QueryError),
    Pump(PumpError),
}

impl From<wifi_connect::WifiErr> for RoutineError {
    fn from(err: wifi_connect::WifiErr) -> Self {
        RoutineError::Wifi(err)
    }
}

impl From<query::QueryError> for RoutineError {
    fn from(err: query::QueryError) -> Self {
        RoutineError::Query(err)
    }
}

impl From<PumpError> for RoutineError {
    fn from(err: PumpError) -> Self {
        RoutineError::Pump(err)
    }
}

fn routine(
    peripherals: Peripherals,
    sys_loop: EspEventLoop<System>,
    nvs: EspNvsPartition<NvsDefault>,
) -> Result<Duration, RoutineError> {
    // WATCHDOG
    // The code restarted automatically after a few seconds because of a watchdog.
    // Setting the timer high worked around it.
    let config = TWDTConfig {
        duration: Duration::from_secs(180),
        panic_on_trigger: true,
        subscribed_idle_tasks: enum_set!(Core::Core0),
    };
    let mut driver = esp_idf_svc::hal::task::watchdog::TWDTDriver::new(peripherals.twdt, &config)
        .map_err(|_| RoutineError::Watchdog)?;
    let _watchdog = driver
        .watch_current_task()
        .map_err(|_| RoutineError::Watchdog)?;

    let pins = peripherals.pins;
    let red = AnyOutputPin::from(pins.gpio12);
    let green1 = AnyOutputPin::from(pins.gpio14);
    let green2 = AnyOutputPin::from(pins.gpio27);
    let green3 = AnyOutputPin::from(pins.gpio26);
    let pump1 = AnyOutputPin::from(pins.gpio25);
    let pump2 = AnyOutputPin::from(pins.gpio32);
    let accu_measure = pins.gpio35.into_ref();

    // Init LED status indicator
    println!("Init LED Signaler");
    let mut led_signaler = StatusSignaler::new(
        red.into_ref(),
        vec![green1.into_ref(), green2.into_ref(), green3.into_ref()],
    );

    println!("Init Accu measure");
    let mut accu = Accu::new(
        peripherals.adc1.into_ref(),
        accu_measure,
        ACCU_VOLTAGE_FACTOR,
        ACCU_CRITICAL_VOLTAGE,
    );
    let accu_volt = accu.measure_volt();
    println!("Accu est: {}V", accu_volt);
    let accu_percent = single_nimh_cell_volt_to_percent(accu_volt / ACCU_NIMH_CELLS_IN_ROW as f32);
    println!("Accu percent measured / calculated: {}%", accu_percent);

    println!("Init Pumps");
    let mut pumps = Pumps::new(
        peripherals.ledc.timer0.into_ref(),
        peripherals.ledc.channel0.into_ref(),
        vec![pump1.into_ref(), pump2.into_ref()],
        accu,
    );

    led_signaler.set_green_number(SIGNAL_WHILE_WIFI);
    let jobs = {
        println!("Connect to Wifi");
        let _wifi =
            connect_to_wifi_with_timeout(Duration::from_secs(10), peripherals.modem, sys_loop, nvs)
                .map_err(RoutineError::from)?;
        led_signaler.set_green_number(SIGNAL_WHILE_FETCH);
        println!("Fetching ESP todos...");
        fetch_jobs(accu_percent)?
    };

    led_signaler.set_green_number(SIGNAL_WHILE_WATERING);
    for job in jobs.watering_jobs.iter() {
        if job.amount_ml == 0 {
            continue;
        }
        match pumps.pump(job.plant_index, job.amount_ml as u32) {
            Some(Err(PumpError::AccuCriticalVoltage)) => {
                println!("Warning! Accu below critical voltage.");
                led_signaler.error_led_on();
                sleep(ERROR_SHOW_RED_LED_DURATION);
                return Ok(Duration::from_secs(jobs.sleep_recommendation_seconds));
            }
            Some(Ok(())) => {}
            None => println!("Warning. No pump connected to {}", job.plant_index),
        }
    }

    // Call destructor to zero all pins, just to be sure
    drop(pumps);

    led_signaler.set_full_green();
    sleep(Duration::from_secs(7));
    Ok(Duration::from_secs(jobs.sleep_recommendation_seconds))
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly.
    // See https://github.com/esp-rs/esp-idf-template/issues/71
    link_patches();
    // Bind the log crate to the ESP Logging facilities
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("Failed to take peripherals");
    let sys_loop = EspSystemEventLoop::take().expect("Failed to take event loop");
    let nvs = EspDefaultNvsPartition::take().expect("Failed to take NVS partition");

    let deepsleep_duration = match routine(peripherals, sys_loop, nvs) {
        Ok(dur) => dur,
        Err(e) => {
            log::error!("Routine failed: {:?}", e);
            ERROR_SLEEP_DURATION
        }
    };

    println!("Sleep now for {:?}", deepsleep_duration);
    deepsleep::deep_sleep(deepsleep_duration);
}
