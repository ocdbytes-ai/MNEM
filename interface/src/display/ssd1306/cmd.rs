use super::SSD1306;
use super::constants;
use crate::display::{DisplayInterface, Result};

impl DisplayInterface for SSD1306 {
    fn flush(&mut self, data: &[u8]) -> Result<()> {
        assert_eq!(
            data.len(),
            constants::FRAMEBUFFER_SIZE,
            "buffer must be exactly {} bytes",
            constants::FRAMEBUFFER_SIZE
        );
        // Reset the pointer to top-left before every frame
        self.i2c.write(&[
            constants::PREFIX_CMD,
            constants::SET_COL_ADDR,
            0,
            constants::WIDTH - 1,
        ])?;
        self.i2c.write(&[
            constants::PREFIX_CMD,
            constants::SET_PAGE_ADDR,
            0,
            constants::PAGES - 1,
        ])?;

        // Send framebuffer in 16-byte chunks
        for chunk in data.chunks(16) {
            let mut buf = [0u8; 17]; // 1 prefix + 16 data max
            buf[0] = constants::PREFIX_DATA;
            buf[1..1 + chunk.len()].copy_from_slice(chunk);
            self.i2c.write(&buf[..1 + chunk.len()])?;
        }
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
        for &byte in constants::INIT_SEQUENCE {
            self.i2c.write(&[constants::PREFIX_CMD, byte])?;
        }
        Ok(())
    }
}

impl SSD1306 {
    pub fn all_on(&mut self) -> Result<()> {
        let buf = [constants::PIXEL_ON; constants::FRAMEBUFFER_SIZE];
        self.flush(&buf)
    }

    pub fn all_off(&mut self) -> Result<()> {
        let buf = [constants::PIXEL_OFF; constants::FRAMEBUFFER_SIZE];
        self.flush(&buf)
    }
}
