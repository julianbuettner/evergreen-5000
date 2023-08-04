use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;

use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};
use std::{
    io::Write,
    net::{Ipv4Addr, TcpListener, TcpStream},
    thread::sleep,
    time::Duration,
};

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: "".into(),
            password: "".into(),
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        sleep(Duration::from_millis(250));
    }

    println!("Should be connected now");
    for _ in 0..10 {
        let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
        println!("IP info: {:?}", ip_info);
        if ip_info.ip != Ipv4Addr::new(0, 0, 0, 0) {
            println!("Got IP!");
            break;
        }
        sleep(Duration::from_millis(250));
    }

    loop {
        sleep(Duration::from_millis(25));
        let mut connector = TcpStream::connect("192.168.0.192:9999");
        if let Err(e) = connector {
            println!("Connection error: {:?}", e);
            continue;
        }
        let mut connector = connector.unwrap();
        let res = connector.write_all(&"NO FUCKING WAY!!!!!\n".as_bytes());
        if let Err(e) = res {
            println!("Write error: {:?}", e);
            continue;
        } else {
            println!("Should have been written!");
        }
    }
}
