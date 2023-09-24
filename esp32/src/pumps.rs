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
pub struct Pumps {}

impl Pumps {
    pub fn new() -> Self {
        let timer_driver = LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &esp_idf_hal::ledc::config::TimerConfig::default().frequency(KiloHertz(10_u32).into()),
        )
        .unwrap();
    }
}
