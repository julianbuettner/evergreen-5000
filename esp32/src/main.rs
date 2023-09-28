use enumset::enum_set;
use esp_idf_hal::{
    cpu::Core, gpio::AnyOutputPin, peripheral::Peripheral, task::watchdog::TWDTConfig,
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{
    eventloop::{EspEventLoop, EspSystemEventLoop, System},
    nvs::{EspDefaultNvsPartition, EspNvsPartition, NvsDefault},
};
use std::{thread::sleep, time::Duration};

use crate::{
    pumps::Pumps, query::fetch_jobs, status_signaler::StatusSignaler,
    wifi_connect::connect_to_wifi_with_timeout,
};

mod deepsleep;
mod pumps;
mod query;
mod status_signaler;
mod wifi_connect;

// Binary on three LEDs
const SIGNAL_WHILE_WIFI: u8 = 1;
const SIGNAL_WHILE_FETCH: u8 = 2;
const SIGNAL_WHILE_WATERING: u8 = 4;

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

    // Init LED status indicator
    println!("Init LED Signaler and pumps");
    let pins = peripherals.pins;
    let mut led_signaler = StatusSignaler::new(
        AnyOutputPin::from(pins.gpio13).into_ref(),
        vec![
            AnyOutputPin::from(pins.gpio12).into_ref(),
            AnyOutputPin::from(pins.gpio14).into_ref(),
            AnyOutputPin::from(pins.gpio27).into_ref(),
        ],
    );
    let mut pumps = Pumps::new(
        peripherals.ledc.timer0.into_ref(),
        peripherals.ledc.channel0.into_ref(),
        vec![
            AnyOutputPin::from(pins.gpio22).into_ref(),
            AnyOutputPin::from(pins.gpio23).into_ref(),
        ],
    );

    led_signaler.set_green_numer(SIGNAL_WHILE_WIFI);
    let jobs = {
        println!("Connect to Wifi");
        let wifi_res =
            connect_to_wifi_with_timeout(Duration::from_secs(10), peripherals.modem, sys_loop, nvs);
        if wifi_res.is_err() {
            led_signaler.error_led_on();
            println!("Could not connect to wifi!");
            sleep(Duration::from_secs(10));
            return Duration::from_secs(3600);
        }
        let _wifi_driver = wifi_res.unwrap();
        led_signaler.set_green_numer(SIGNAL_WHILE_FETCH);
        println!("Fetching ESP todos...");
        fetch_jobs(100.) // drop and disconnect wifi
    };
    if let Err(e) = jobs {
        println!("Error fetching jobs: {:?}", e);
        led_signaler.error_led_on();
        sleep(Duration::from_secs(10));
        return Duration::from_secs(3600);
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
    sleep(Duration::from_secs(2));
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

    deepsleep::deep_sleep(deepsleep_duration);
}
