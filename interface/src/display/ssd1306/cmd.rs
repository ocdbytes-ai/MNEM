use super::SSD1306;
use super::constants;
use crate::display::{DisplayInterface, Error, Result};

impl DisplayInterface for SSD1306 {
    fn width(&self) -> usize {
        constants::WIDTH as usize
    }

    fn height(&self) -> usize {
        constants::HEIGHT as usize
    }

    fn buffer_size(&self) -> usize {
        constants::FRAMEBUFFER_SIZE
    }

    fn flush(&mut self, data: &[u8]) -> Result<()> {
        if data.len() != constants::FRAMEBUFFER_SIZE {
            return Err(Error::InvalidBufferSize {
                expected: constants::FRAMEBUFFER_SIZE,
                got: data.len(),
            });
        }

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
