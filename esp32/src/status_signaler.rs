use esp_idf_hal::gpio::{Output, OutputPin, PinDriver};

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

pub struct StatusSignaler<
    'a,
    Red: OutputPin,
    Green1: OutputPin,
    Green2: OutputPin,
    Green3: OutputPin,
    Green4: OutputPin,
> {
    red: PinDriver<'a, Red, Output>,
    green1: PinDriver<'a, Green1, Output>,
    green2: PinDriver<'a, Green2, Output>,
    green3: PinDriver<'a, Green3, Output>,
    green4: PinDriver<'a, Green4, Output>,
}

impl<
        'a,
        Red: OutputPin,
        Green1: OutputPin,
        Green2: OutputPin,
        Green3: OutputPin,
        Green4: OutputPin,
    > StatusSignaler<'a, Red, Green1, Green2, Green3, Green4>
{
    pub fn new(
        red: Red,
        green1: Green1,
        green2: Green2,
        green3: Green3,
        green4: Green4,
    ) -> StatusSignaler<'a, Red, Green1, Green2, Green3, Green4> {
        Self {
            red: PinDriver::output(red).unwrap(),
            green1: PinDriver::output(green1).unwrap(),
            green2: PinDriver::output(green2).unwrap(),
            green3: PinDriver::output(green3).unwrap(),
            green4: PinDriver::output(green4).unwrap(),
        }
    }

    pub fn error_led_on(&mut self) {
        self.red.set_high().unwrap()
    }

    pub fn error_led_off(&mut self) {
        self.red.set_low().unwrap()
    }

    pub fn set_green_numer(&mut self, num: u8) {
        let (g1, g2, g3, g4) = (
            (num >> 0) % 2 == 1,
            (num >> 1) % 2 == 1,
            (num >> 2) % 2 == 1,
            (num >> 3) % 2 == 1,
        );
        match g1 {
            true => self.green1.set_high().unwrap(),
            false => self.green1.set_low().unwrap(),
        }
        match g2 {
            true => self.green2.set_high().unwrap(),
            false => self.green2.set_low().unwrap(),
        }
        match g3 {
            true => self.green3.set_high().unwrap(),
            false => self.green3.set_low().unwrap(),
        }
        match g4 {
            true => self.green4.set_high().unwrap(),
            false => self.green4.set_low().unwrap(),
        }
    }

    pub fn set_full_green(&mut self) {
        self.set_green_numer(u8::MAX)
    }
}
