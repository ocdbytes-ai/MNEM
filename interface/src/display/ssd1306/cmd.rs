use super::SSD1306;
use super::constants;
use crate::display::{DisplayInterface, Error, Mono, Result};

impl DisplayInterface for SSD1306 {
    type Pixel = Mono;

    fn width(&self) -> usize {
        constants::WIDTH as usize
    }

    fn height(&self) -> usize {
        constants::HEIGHT as usize
    }

    fn encode_pixel(&self, pixel: &Mono, buf: &mut [u8], x: usize, y: usize) {
        assert!(x < self.width() && y < self.height());
        // SSD1306 page-based layout: each byte covers 8 vertical pixels in a column
        let byte = x + (y / 8) * self.width();
        let bit = y % 8;
        if pixel.0 {
            buf[byte] |= 1 << bit;
        } else {
            buf[byte] &= !(1 << bit);
        }
    }

    fn flush(&mut self, data: &[u8]) -> Result<()> {
        let expected = self.buffer_size();
        if data.len() != expected {
            return Err(Error::InvalidBufferSize {
                expected,
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
