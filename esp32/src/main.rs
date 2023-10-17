#![feature(exclusive_range_pattern)]
use enumset::enum_set;
use esp_idf_hal::{
    cpu::Core, gpio::AnyOutputPin, peripheral::Peripheral, task::watchdog::TWDTConfig,
};
use esp_idf_sys::{self as _};

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    eventloop::{EspEventLoop, EspSystemEventLoop, System},
    nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault},
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
const ACCU_CIRITICAL_VOLTAGE: f32 = 4.0; // stop watering below critical volt (under load)

// measure how much the pump outputs continously
// over 10s. Measure the result and divide by 10s.
// Measure the voltage of the battery while pumping.
// Divide previously value by volt.
// I measured 1300ml / 10s / 5.8V
pub const PUMP_ML_PER_VOLT_SECOND: f32 = 22.413793;
pub const PUMP_WARMUP_MS: u32 = 25; // no abrupt start
pub const PUMP_COOLDOWN_MS: u32 = 25; // no abrupt stop
                                      // Hard coded maximum, which will never by exceeded
pub const PUMP_MAX_PUMP_DURATION: Duration = Duration::from_secs(15);

// When returning routine, the destructors
// of pumps and led signal should set every
// pin to low.
fn routine(
    peripherals: Peripherals,
    sys_loop: EspEventLoop<System>,
    nvs: EspNvsPartition<NvsDefault>,
) -> Duration {
    // WATCHDOG
    // So, the code restarted automatically after a few seconds
    // because of some watchdog thing. I could not `feed` it,
    // it stopped anyways. But settings the timer high worked somehow.
    // So it's not 3 min and I don't touch it. Help is appreciated.
    let config = TWDTConfig {
        duration: Duration::from_secs(180),
        panic_on_trigger: true,
        subscribed_idle_tasks: enum_set!(Core::Core0),
    };
    let mut driver =
        esp_idf_hal::task::watchdog::TWDTDriver::new(peripherals.twdt, &config).unwrap();
    let _watchdog = driver.watch_current_task().unwrap();

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
        ACCU_CIRITICAL_VOLTAGE,
    );
    let accu_volt = accu.measure_volt();
    println!("Accu est: {}V", accu_volt);
    let accu_percent = single_nimh_cell_volt_to_percent(accu_volt / ACCU_NIMH_CELLS_IN_ROW as f32);
    println!("Accu percent measures / calculated: {}%", accu_percent);
    println!("Init Pumps");
    let mut pumps = Pumps::new(
        peripherals.ledc.timer0.into_ref(),
        peripherals.ledc.channel0.into_ref(),
        vec![pump1.into_ref(), pump2.into_ref()],
        accu,
    );

    led_signaler.set_green_numer(SIGNAL_WHILE_WIFI);
    let jobs = {
        println!("Connect to Wifi");
        let wifi_res =
            connect_to_wifi_with_timeout(Duration::from_secs(10), peripherals.modem, sys_loop, nvs);
        if wifi_res.is_err() {
            led_signaler.error_led_on();
            println!("Could not connect to wifi!");
            sleep(ERROR_SHOW_RED_LED_DURATION);
            return ERROR_SLEEP_DURATION;
        }
        let _wifi_driver = wifi_res.unwrap();
        led_signaler.set_green_numer(SIGNAL_WHILE_FETCH);
        println!("Fetching ESP todos...");
        fetch_jobs(accu_percent) // drop and disconnect wifi
    };
    if let Err(e) = jobs {
        println!("Error fetching jobs: {:?}", e);
        led_signaler.error_led_on();
        sleep(ERROR_SHOW_RED_LED_DURATION);
        return ERROR_SLEEP_DURATION;
    }
    let jobs = jobs.unwrap();
    led_signaler.set_green_numer(SIGNAL_WHILE_WATERING);
    for job in jobs.watering_jobs.iter() {
        if job.amount_ml == 0 {
            continue;
        }
        match pumps.pump(job.plant_index, job.amount_ml as u32) {
            Some(Err(PumpError::AccuCriticalVoltage)) => {
                println!("Warning! Accu below critical voltage.");
                led_signaler.error_led_on();
                sleep(ERROR_SHOW_RED_LED_DURATION);
                return Duration::from_secs(jobs.sleep_recommendation_seconds);
            }
            Some(Ok(())) => {}
            None => println!("Warning. No pump connected to {}", job.plant_index),
        }
    }
    // Call destructor to zero all pins, just to be sure
    drop(pumps);

    led_signaler.set_full_green();
    sleep(Duration::from_secs(7));
    Duration::from_secs(jobs.sleep_recommendation_seconds)
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let deepsleep_duration = routine(peripherals, sys_loop, nvs);

    println!("Sleep now for {:?}", deepsleep_duration);

    deepsleep::deep_sleep(deepsleep_duration);
}
