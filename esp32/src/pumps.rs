use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use esp_idf_hal::{
    gpio::{ADCPin, AnyOutputPin, PinDriver},
    ledc::{LedcDriver, LedcTimerDriver, CHANNEL0, TIMER0},
    peripheral::PeripheralRef,
    units::KiloHertz,
};

use crate::{
    accu::Accu, ACCU_CIRITICAL_VOLTAGE, PUMP_COOLDOWN_MS, PUMP_MAX_PUMP_DURATION,
    PUMP_ML_PER_VOLT_SECOND, PUMP_WARMUP_MS,
};

pub enum PumpError {
    AccuCriticalVoltage,
}

pub struct Pumps<'a, A: ADCPin> {
    timer0: PeripheralRef<'a, TIMER0>,
    channel0: PeripheralRef<'a, CHANNEL0>,
    pumps: Vec<PeripheralRef<'a, AnyOutputPin>>,
    accu: Accu<'a, A>,
}

impl<A: ADCPin> Drop for Pumps<'_, A> {
    fn drop(&mut self) {
        for p in self.pumps.iter_mut() {
            let mut p = PinDriver::output(p.reborrow()).unwrap();
            p.set_low().unwrap();
        }
    }
}

impl<'a, A: ADCPin> Pumps<'a, A> {
    pub fn new(
        timer0: PeripheralRef<'a, TIMER0>,
        channel0: PeripheralRef<'a, CHANNEL0>,
        mut pumps: Vec<PeripheralRef<'a, AnyOutputPin>>,
        accu: Accu<'a, A>,
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
            accu,
        }
    }

    pub fn pump(&mut self, index: usize, amount_ml: u32) -> Option<Result<(), PumpError>> {
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
        for ms in 0..PUMP_WARMUP_MS {
            driver.set_duty(max_duty * ms / PUMP_WARMUP_MS).unwrap();
            sleep(Duration::from_millis(1));
        }

        // linear start/stop => integral 0.5
        let mut ml_watered: f32 = {
            let volt = self.accu.measure_volt();
            let warmup_cooldown_sec = (PUMP_WARMUP_MS + PUMP_COOLDOWN_MS) as f32 / 1000.;
            PUMP_ML_PER_VOLT_SECOND * volt * warmup_cooldown_sec * 0.5
        };

        println!("Max pump now");
        driver.set_duty(max_duty).unwrap();
        let start = Instant::now();
        let mut delta = Instant::now();
        while start.elapsed() < PUMP_MAX_PUMP_DURATION && (ml_watered as u32) < amount_ml {
            sleep(Duration::from_millis(5));
            let volt = self.accu.measure_volt();
            if volt < self.accu.get_critical_volt() {
                driver.set_duty(0).unwrap();
                return Some(Err(PumpError::AccuCriticalVoltage));
            }
            let delta_watered = PUMP_ML_PER_VOLT_SECOND * volt * delta.elapsed().as_secs_f32();
            ml_watered += delta_watered;
            delta = Instant::now();
        }

        println!("Slowly stop pump again");
        // Slowly stop again
        for ms in 0..PUMP_COOLDOWN_MS {
            driver
                .set_duty(max_duty * (PUMP_COOLDOWN_MS - ms) / PUMP_COOLDOWN_MS)
                .unwrap();
            sleep(Duration::from_millis(1));
        }
        println!("Done pumping: wateres {}ml in {}ms", ml_watered, start.elapsed().as_millis());
        Some(Ok(()))
    }
}
