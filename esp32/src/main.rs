use enumset::{EnumSet, enum_set};
use esp_idf_hal::{units::KiloHertz, task::watchdog::TWDTConfig, cpu::Core};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::{
    ledc::{LedcDriver, LedcTimerDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use std::{
    io::Write,
    net::{Ipv4Addr, TcpListener, TcpStream},
    thread::sleep,
    time::Duration,
};

use crate::{
    deepsleep::deep_sleep, query::content, status_signaler::StatusSignaler,
    wifi_connect::connect_to_wifi_with_timeout,
};
use embedded_hal::digital::v2::OutputPin;
use esp_idf_hal::gpio::PinDriver;

mod deepsleep;
mod query;
mod status_signaler;
mod pumps;
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
        subscribed_idle_tasks: enum_set!(Core::Core0)
    };
    let mut driver = esp_idf_hal::task::watchdog::TWDTDriver::new(peripherals.twdt, &config).unwrap();
    let mut watchdog = driver.watch_current_task().unwrap();

    // Init LED status indicator
    println!("Init LED Signaler");
    let pins = peripherals.pins;
    let mut led_signaler = StatusSignaler::new(
        pins.gpio27, // red
        // 2 onboard, make to 26
        pins.gpio2, pins.gpio25, pins.gpio33, pins.gpio32,
    );
    led_signaler.set_green_numer(1);

    println!("Connect to Wifi");
    let wifi_res =
        connect_to_wifi_with_timeout(Duration::from_secs(10), peripherals.modem, sys_loop, nvs);
    if wifi_res.is_err() {
        panic!("Could not connect to wifi!");
    }
    let mut wifi_driver = wifi_res.unwrap();

    let timer_driver = LedcTimerDriver::new(
        peripherals.ledc.timer0,
        &esp_idf_hal::ledc::config::TimerConfig::default().frequency(KiloHertz(10_u32).into()),
    )
    .unwrap();
    let mut driver = LedcDriver::new(peripherals.ledc.channel0, timer_driver, pins.gpio23).unwrap();
    let max_duty = driver.get_max_duty();

    let steps = 999;
    for _ in 0..999 {
        for i in 0..steps {
            driver.set_duty(max_duty * i / steps).unwrap();
            sleep(Duration::from_millis(2));
        }
    }

    println!("Going deep sleep");
    // wifi_driver.disconnect().unwrap();
    deepsleep::deep_sleep(Duration::from_secs(2));
}
