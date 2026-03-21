use std::{thread::sleep, time::Duration};

use interface::display::{DisplayInterface, SSD1306};
use rppal::i2c::I2c;

fn main() {
    let i2c = I2c::new().expect("failed to initiate i2c");
    let mut display = SSD1306::new(i2c).expect("failed to set i2c address");
    display.init().expect("failed to initialize display");

    display.all_on().expect("failed to turn on all pixels");
    sleep(Duration::from_secs(2));
    display.all_off().expect("failed to turn off all pixels");
}
