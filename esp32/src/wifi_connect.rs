use std::{
    net::Ipv4Addr,
    thread::sleep,
    time::{Duration, Instant},
};

use embedded_svc::wifi::{ClientConfiguration as WifiClientConfiguration, Configuration};
use esp_idf_hal::modem::Modem;
use esp_idf_svc::{
    eventloop::{EspEventLoop, System},
    nvs::{EspNvsPartition, NvsDefault},
    wifi::EspWifi,
};

#[derive(Clone, Debug)]
pub enum WifiErr {
    TimeoutConnect,
    TimeoutIp,
    //WrongCredentials,
}

pub struct WifiConnection<'a> {
    wifi_driver: EspWifi<'a>,
}

impl Drop for WifiConnection<'_> {
    fn drop(&mut self) {
        match self.wifi_driver.disconnect() {
            Err(_) => println!("Failed to disconnect wifi!"),
            _ => (),
        }
    }
}

pub fn connect_to_wifi_with_timeout(
    timeout: Duration,
    modem: Modem,
    sys_loop: EspEventLoop<System>,
    nvs: EspNvsPartition<NvsDefault>,
) -> Result<WifiConnection<'static>, WifiErr> {
    let mut wifi_driver = EspWifi::new(modem, sys_loop, Some(nvs)).unwrap();
    wifi_driver
        .set_configuration(&Configuration::Client(WifiClientConfiguration {
            ssid: env!("WIFI_SSID").into(),
            password: env!("WIFI_PASS").into(),
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    let task_start = Instant::now();
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config);
        if task_start.elapsed() > timeout {
            return Err(WifiErr::TimeoutConnect);
        }
        sleep(Duration::from_millis(250));
    }
    println!("Connected to wifi!");

    loop {
        let ip_info = wifi_driver.sta_netif().get_ip_info().unwrap();
        if ip_info.ip != Ipv4Addr::new(0, 0, 0, 0) {
            println!("Got IP!");
            break;
        }
        if task_start.elapsed() > timeout {
            return Err(WifiErr::TimeoutIp);
        }
        sleep(Duration::from_millis(150));
    }

    Ok(WifiConnection { wifi_driver })
}
