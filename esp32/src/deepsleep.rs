use std::time::Duration;

pub fn deep_sleep(dur: Duration) -> ! {
    println!("Entering deep sleep...");
    unsafe {
        esp_idf_sys::esp_deep_sleep(dur.as_micros() as u64);
    }
}
