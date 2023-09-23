use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::peripherals::Peripherals;
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

    // Init LED status indicator
    let pins = peripherals.pins;
    let mut led_signaler = StatusSignaler::new(
        pins.gpio4, // red
        pins.gpio5, pins.gpio6, pins.gpio7, pins.gpio8,
    );
    let mut onboard_led = PinDriver::output(pins.gpio2).unwrap();
    onboard_led.set_high().unwrap();
    led_signaler.set_green_numer(1);

    let wifi_res =
        connect_to_wifi_with_timeout(Duration::from_secs(10), peripherals.modem, sys_loop, nvs);
    if wifi_res.is_err() {
        panic!("Could not connect to wifi!");
    }
    let mut wifi_driver = wifi_res.unwrap();

    let mut toggle = true;
    onboard_led.set_high().unwrap();
    sleep(Duration::from_millis(1000));
    onboard_led.set_low().unwrap();

    for _ in 0..10 {
        sleep(Duration::from_millis(25));
        match toggle {
            true => onboard_led.set_high().unwrap(),
            false => onboard_led.set_low().unwrap(),
        };
        toggle = !toggle;
        let mut connector = TcpStream::connect("192.168.0.192:9999");
        if let Err(e) = connector {
            println!("Connection error: {:?}", e);
            continue;
        }
        let mut connector = connector.unwrap();
        let message = content();
        let res = connector.write_all(&message.as_bytes());
        if let Err(e) = res {
            println!("Write error: {:?}", e);
            continue;
        } else {
            println!("Should have been written!");
        }
    }

    println!("Going deep sleep");
    wifi_driver.disconnect().unwrap();
    deepsleep::deep_sleep(Duration::from_secs(10));
}
