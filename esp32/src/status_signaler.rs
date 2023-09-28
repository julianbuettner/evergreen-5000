use esp_idf_hal::{
    gpio::{AnyOutputPin, Output, PinDriver},
    peripheral::PeripheralRef,
};

/// To signal what the box containing the
/// ESP32 is doing, a few LEDs should be installed and pointed
/// towards the outside.
/// There will be multiple green LEDs in a row and one red LED.
/// We can now signal numbers by using the green LEDs as
/// binary digits and the red LED to signal if
/// we signal a "good state" or a "bad state".
///
/// To signal progress the following states are defined:
///
/// - 1. Turning on
/// - 2. Connecting to Wifi
/// - 3. Getting job from the server
/// - 4. Doing watering if tasked
/// - 5. Being successful, terminating after 1s.
///
/// 5 binary is 0b1001, so
/// our LEDs should light so:  [1, 0, 0, 1]
///
/// If a step did not succeed, the red
/// LED can be turned on.

pub struct StatusSignaler<'a> {
    red_driver: PinDriver<'a, AnyOutputPin, Output>,
    // red: PeripheralRef<'a, AnyOutputPin>,
    green_driver: Vec<PinDriver<'a, AnyOutputPin, Output>>,
}

impl Drop for StatusSignaler<'_> {
    fn drop(&mut self) {
        self.off();
    }
}

impl<'a> StatusSignaler<'a> {
    pub fn new(
        red: PeripheralRef<'a, AnyOutputPin>,
        green: Vec<PeripheralRef<'a, AnyOutputPin>>,
    ) -> StatusSignaler<'a> {
        let mut red_driver = PinDriver::output(red).unwrap();
        red_driver.set_low().unwrap();
        let mut green_driver: Vec<PinDriver<AnyOutputPin, Output>> = green
            .into_iter()
            .map(|p| PinDriver::output(p).unwrap())
            .collect();
        // Ensure pins are low
        for g in green_driver.iter_mut() {
            g.set_low().unwrap();
        }
        Self {
            red_driver,
            green_driver,
        }
    }

    pub fn error_led_on(&mut self) {
        self.red_driver.set_high().unwrap();
    }

    pub fn set_green_numer(&mut self, num: u8) {
        for (i, pin) in self.green_driver.iter_mut().enumerate() {
            match (num >> i) % 2 == 1 {
                true => pin.set_high().unwrap(),
                false => pin.set_low().unwrap(),
            }
        }
    }

    pub fn set_full_green(&mut self) {
        for pin in self.green_driver.iter_mut() {
            pin.set_high().unwrap();
        }
    }

    pub fn off(&mut self) {
        for p in self.green_driver.iter_mut() {
            p.set_low().unwrap();
        }
        self.red_driver.set_low().unwrap();
    }
}
