mod cmd;
pub(crate) mod constants;

use crate::display::Result;
use rppal::i2c::I2c;

pub struct SSD1306 {
    i2c: I2c,
}

impl SSD1306 {
    pub const I2C_ADDR: u16 = constants::I2C_ADDR as u16;

    pub fn new(mut i2c: I2c) -> Result<Self> {
        i2c.set_slave_address(Self::I2C_ADDR)?;
        Ok(Self { i2c })
    }
}
