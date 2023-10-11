use std::thread::sleep;
use std::time::Duration;

use esp_idf_hal::adc::*;

use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::{
    adc::{config::Config, AdcChannelDriver, AdcDriver, ADC1},
    gpio::{ADCPin, AnyOutputPin, PinDriver},
    peripheral::PeripheralRef,
};

const LOW_VOLT: u32 = 142; // pin grounded
const HIGH_VOLT: u32 = 3129; // pin connected to 3.3

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

pub struct Accu<'a, A: ADCPin> {
    adc2: PeripheralRef<'a, ADC1>,
    voltage_pin: PeripheralRef<'a, A>,
    factor: f32,
    min_volt: f32,
}

impl<'a, A: ADCPin> Accu<'a, A> {
    pub fn new(
        adc2: PeripheralRef<'a, ADC1>,
        voltage_pin: PeripheralRef<'a, A>,
        factor: f32,
        min_volt: f32,
    ) -> Self {
        Self {
            adc2,
            voltage_pin,
            factor,
            min_volt,
        }
    }
    pub fn measure_volt(&mut self) -> f32 {
        let mut adc =
            AdcDriver::new(self.adc2.reborrow(), &Config::new().calibration(true)).unwrap();
        let mut adc_pin: esp_idf_hal::adc::AdcChannelDriver<A, Atten11dB<_>> =
            AdcChannelDriver::new(self.voltage_pin.reborrow()).unwrap();

        const SAMPLE_SIZE: usize = 30;
        let mut samples: [u16; SAMPLE_SIZE] = [0; SAMPLE_SIZE];
        for i in 0..SAMPLE_SIZE {
            samples[i] = adc.read(&mut adc_pin).unwrap();
            sleep(Duration::from_micros(250)); // 0.25m
        }
        let raw_value: f32 = samples.iter().map(|v| *v as f32).sum::<f32>() / SAMPLE_SIZE as f32;

        println!("Raw value: {}", raw_value);
        let volt_measured = 3.3 * (raw_value - LOW_VOLT as f32) / (HIGH_VOLT) as f32;
        println!("Measured Accu Voltage: {}", volt_measured);
        println!("Scaled Accu Voltage: {}", volt_measured * self.factor);
        volt_measured * self.factor
    }
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
