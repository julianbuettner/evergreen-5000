use enumset::enum_set;
use esp_idf_hal::{
    cpu::Core, gpio::AnyOutputPin, peripheral::Peripheral, task::watchdog::TWDTConfig,
    units::KiloHertz,
};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::{
    ledc::{LedcDriver, LedcTimerDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use std::{thread::sleep, time::Duration};

use crate::{
    deepsleep::deep_sleep, pumps::Pumps, query::fetch_jobs, status_signaler::StatusSignaler,
    wifi_connect::connect_to_wifi_with_timeout,
};

mod deepsleep;
mod pumps;
mod query;
mod status_signaler;
mod wifi_connect;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

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
    println!("Init LED Signaler");
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

    led_signaler.set_green_numer(1);
    let jobs = {
        println!("Connect to Wifi");
        let wifi_res =
            connect_to_wifi_with_timeout(Duration::from_secs(10), peripherals.modem, sys_loop, nvs);
        if wifi_res.is_err() {
            led_signaler.error_led_on();
            println!("Could not connect to wifi!");
            sleep(Duration::from_secs(10));
            deep_sleep(Duration::from_secs(3600));
        }
        let _wifi_driver = wifi_res.unwrap();

        led_signaler.set_green_numer(2);
        println!("Fetching ESP todos...");
        fetch_jobs(100.) // drop and disconnect wifi
    };
    if let Err(e) = jobs {
        println!("Error fetching jobs: {:?}", e);
        led_signaler.error_led_on();
        sleep(Duration::from_secs(10));
        drop(led_signaler);
        deep_sleep(Duration::from_secs(900));
    }
    let jobs = jobs.unwrap();

    led_signaler.set_green_numer(3);
    for (index, duration) in jobs.plantings.iter().enumerate() {
        if duration.is_zero() {
            continue;
        }
        match pumps.pump(index, duration.clone()) {
            Some(_) => (),
            None => println!("Warning. No pump connected to {}", index),
        }
    }

    println!("Going deep sleep");
    led_signaler.set_full_green();
    sleep(Duration::from_secs(1));
    // wifi_driver.disconnect().unwrap();
    deepsleep::deep_sleep(jobs.sleep_recommendation);
}
