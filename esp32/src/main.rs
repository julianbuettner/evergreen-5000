#![feature(exclusive_range_pattern)]
use enumset::enum_set;
use esp_idf_hal::{
    cpu::Core,
    gpio::{AnyIOPin, AnyOutputPin, PinDriver},
    peripheral::{Peripheral, PeripheralRef},
    task::watchdog::TWDTConfig,
};
use esp_idf_sys::{self as _, gpio_num_t_GPIO_NUM_32}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    eventloop::{EspEventLoop, EspSystemEventLoop, System},
    nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault},
};
use std::{thread::sleep, time::Duration};

use crate::{
    accu::{measure_accu, single_nimh_cell_volt_to_percent},
    pumps::Pumps,
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

const NIMH_CELLS_IN_ROW: usize = 8;
const ACCU_VOLTAGE_FACTOR: f32 = 1330. / 330.;

// When returning routine, the destructors
// of pumps and led signal should set every
// pin to low.
fn routine(
    peripherals: Peripherals,
    sys_loop: EspEventLoop<System>,
    nvs: EspNvsPartition<NvsDefault>,
) -> (Duration, Vec<AnyOutputPin>) {
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
    let red = AnyOutputPin::from(pins.gpio13);
    let green1 = AnyOutputPin::from(pins.gpio12);
    let green2 = AnyOutputPin::from(pins.gpio14);
    let green3 = AnyOutputPin::from(pins.gpio27);
    let pump1 = AnyOutputPin::from(pins.gpio22);
    let pump2 = AnyOutputPin::from(pins.gpio23);
    let accu_measure_controller = AnyOutputPin::from(pins.gpio32);

    // Init LED status indicator
    println!("Init LED Signaler");
    let mut led_signaler = StatusSignaler::new(
        red.into_ref(),
        vec![green1.into_ref(), green2.into_ref(), green3.into_ref()],
    );
    println!("Init Pumps");
    let mut pumps = Pumps::new(
        peripherals.ledc.timer0.into_ref(),
        peripherals.ledc.channel0.into_ref(),
        vec![pump1.into_ref(), pump2.into_ref()],
    );
    println!("Init Accu measure");
    let accu_volt = measure_accu(
        peripherals.adc1.into_ref(),
        accu_measure_controller.into_ref(),
        pins.gpio35.into_ref(),
    ) * ACCU_VOLTAGE_FACTOR;
    let accu_percent = single_nimh_cell_volt_to_percent(accu_volt / NIMH_CELLS_IN_ROW as f32);
    println!("Accu percent measures / calculated: {}%", accu_percent);
    return (Duration::from_secs(5), vec![accu_measure_controller]);

    led_signaler.set_green_numer(SIGNAL_WHILE_WIFI);
    let jobs = {
        println!("Connect to Wifi");
        let wifi_res =
            connect_to_wifi_with_timeout(Duration::from_secs(10), peripherals.modem, sys_loop, nvs);
        if wifi_res.is_err() {
            led_signaler.error_led_on();
            println!("Could not connect to wifi!");
            sleep(Duration::from_secs(10));
            return (Duration::from_secs(3600), vec![accu_measure_controller]);
        }
        let _wifi_driver = wifi_res.unwrap();
        led_signaler.set_green_numer(SIGNAL_WHILE_FETCH);
        println!("Fetching ESP todos...");
        fetch_jobs(accu_percent) // drop and disconnect wifi
    };
    if let Err(e) = jobs {
        println!("Error fetching jobs: {:?}", e);
        led_signaler.error_led_on();
        sleep(Duration::from_secs(10));
        return (Duration::from_secs(3600), vec![accu_measure_controller]);
    }
    let jobs = jobs.unwrap();
    led_signaler.set_green_numer(SIGNAL_WHILE_WATERING);
    for job in jobs.watering_jobs.iter() {
        if job.amount_ml == 0 {
            continue;
        }
        match pumps.pump(job.plant_index, job.amount_ml as u32) {
            Some(_) => (),
            None => println!("Warning. No pump connected to {}", job.plant_index),
        }
    }
    // Call destructor to zero all pins, just to be sure
    drop(pumps);

    led_signaler.set_full_green();
    sleep(Duration::from_secs(7));
    (
        Duration::from_secs(jobs.sleep_recommendation_seconds),
        vec![accu_measure_controller],
    )
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

    let (deepsleep_duration, low_pins) = routine(peripherals, sys_loop, nvs);

    println!("Sleep now for {:?}", deepsleep_duration);

    let mut drivers = Vec::new();
    for pin in low_pins {
        let mut d = PinDriver::output(pin).unwrap();
        d.set_low().unwrap();
        drivers.push(d);
    }
    unsafe {
        esp_idf_sys::gpio_hold_en(gpio_num_t_GPIO_NUM_32);
        esp_idf_sys::gpio_deep_sleep_hold_en();
    }

    deepsleep::deep_sleep(deepsleep_duration);
    drop(drivers)
}
