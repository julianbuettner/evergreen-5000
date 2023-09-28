use std::{thread::sleep, time::Duration};

use esp_idf_hal::{
    gpio::{AnyOutputPin, PinDriver},
    ledc::{LedcDriver, LedcTimerDriver, CHANNEL0, TIMER0},
    peripheral::PeripheralRef,
    units::KiloHertz,
};

pub struct Pumps<'a> {
    timer0: PeripheralRef<'a, TIMER0>,
    channel0: PeripheralRef<'a, CHANNEL0>,
    pumps: Vec<PeripheralRef<'a, AnyOutputPin>>,
}

impl Drop for Pumps<'_> {
    fn drop(&mut self) {
        for p in self.pumps.iter_mut() {
            let mut p = PinDriver::output(p.reborrow()).unwrap();
            p.set_low().unwrap();
        }
    }
}

impl<'a> Pumps<'a> {
    pub fn new(
        timer0: PeripheralRef<'a, TIMER0>,
        channel0: PeripheralRef<'a, CHANNEL0>,
        mut pumps: Vec<PeripheralRef<'a, AnyOutputPin>>,
    ) -> Self {
        // Esnure pins are low
        for p in pumps.iter_mut() {
            let mut p = PinDriver::output(p.reborrow()).unwrap();
            p.set_low().unwrap();
        }
        Self {
            timer0,
            channel0,
            pumps,
        }
    }

    pub fn pump(&mut self, index: usize, amount_ml: u32) -> Option<()> {
        let pump = self.pumps.get_mut(index)?;

        let timer_driver = LedcTimerDriver::new(
            self.timer0.reborrow(),
            &esp_idf_hal::ledc::config::TimerConfig::default().frequency(KiloHertz(10_u32).into()),
        )
        .unwrap();
        let mut driver =
            LedcDriver::new(self.channel0.reborrow(), timer_driver, pump.reborrow()).unwrap();
        let max_duty = driver.get_max_duty();

        // Slowly start pump within 100ms
        println!("Slowly starting pump...");
        for ms in 0..100 {
            driver.set_duty(max_duty * ms / 100).unwrap();
            sleep(Duration::from_millis(1));
        }
        println!("Max pump now");
        driver.set_duty(max_duty).unwrap();
        sleep(amount_ml * Duration::from_millis(5));
        println!("Slowly stop pump again");
        // Slowly stop again
        for ms in 0..1000 {
            driver.set_duty(max_duty * (1000 - ms) / 1000).unwrap();
            sleep(Duration::from_millis(1));
        }
        println!("Done pumping!");
        Some(())
    }
}
