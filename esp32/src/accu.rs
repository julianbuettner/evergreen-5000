use std::thread::sleep;
use std::time::Duration;

use esp_idf_hal::adc::*;

use esp_idf_hal::{
    adc::{config::Config, AdcChannelDriver, AdcDriver, ADC2},
    gpio::{ADCPin, AnyOutputPin, PinDriver},
    peripheral::PeripheralRef,
};

const LOW_VOLT: u32 = 128; // pin grounded
const HIGH_VOLT: u32 = 3116; // pin connected to 3.3

pub fn single_nimh_cell_volt_to_percent(volt: f32) -> f32 {
    let curve = [
        (0.0, -100.),
        (0.8, -30.),
        (1.0, 0.),
        (1.1, 5.),
        (1.15, 10.),
        (1.18, 20.),
        (1.2, 50.),
        (1.23, 80.),
        (1.3, 95.),
        (1.4, 120.),
        (1.6, 200.),
    ];
    for i in 1..curve.len() {
        let (volt_a, percent_a) = curve[i - 1];
        let (volt_b, percent_b) = curve[i];
        if volt_a <= volt && volt <= volt_b {
            let c = (volt - volt_a) / (volt_b - volt_a);
            return percent_a + c * (percent_b - percent_a);
        }
    }
    if volt > curve.last().unwrap().0 {
        return 999.;
    }
    if volt < curve.first().unwrap().0 {
        return -99.;
    }
    panic!("Should never be reached.");
}

pub fn measure_accu<'a, A: ADCPin>(
    adc2: PeripheralRef<'a, ADC2>,
    controller_pin: PeripheralRef<'a, AnyOutputPin>,
    voltage_pin: PeripheralRef<'a, A>,
    factor: f32, // voltage divider, e.g. 330/1330
) -> f32 {
    if factor > 1.0 {
        println!("Warning! Voltage divider value must be 1.0 or less.");
    }
    let mut adc = AdcDriver::new(adc2, &Config::new().calibration(true)).unwrap();
    let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<A, Atten11dB<_>> =
        AdcChannelDriver::new(voltage_pin).unwrap();

    // activate measuring:
    let mut controller = PinDriver::output(controller_pin).unwrap();
    controller.set_high().unwrap();
    sleep(Duration::from_millis(5));

    //  loop {
    //     std::thread::sleep(Duration::from_millis(250));
    //     println!("ADC value: {}", adc.read(&mut adc_pin).unwrap());
    //  }
    let raw_value = adc.read(&mut adc_pin).unwrap();
    controller.set_low().unwrap();

    let volt_measured = 3.3 * raw_value as f32 / (HIGH_VOLT - LOW_VOLT) as f32;

    volt_measured / factor
}

#[cfg(test)]
mod test {
    use super::*;

    fn feq(a: f32, b: f32) -> bool {
        let x = a / b;
        (x > 0.99) && (x < 1.01)
    }

    #[test]
    fn accu_percent() {
        assert!(feq(single_nimh_cell_volt_to_percent(1.2), 40.))
    }
}
