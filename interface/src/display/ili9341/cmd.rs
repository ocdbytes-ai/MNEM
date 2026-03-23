use super::ILI9341Display;
use crate::display::{DisplayInterface, Error, Result, Rgb565};
use display_interface::WriteOnlyDataCommand;
use embedded_hal::digital::OutputPin;

impl<IFACE, RESET> DisplayInterface for ILI9341Display<IFACE, RESET>
where
    IFACE: WriteOnlyDataCommand,
    RESET: OutputPin,
{
    type Pixel = Rgb565;

    fn width(&self) -> usize {
        self.inner.width()
    }

    fn height(&self) -> usize {
        self.inner.height()
    }

    fn encode_pixel(&self, pixel: &Rgb565, buf: &mut [u8], x: usize, y: usize) {
        assert!(x < self.width() && y < self.height());
        let offset = (y * self.width() + x) * 2;
        let bytes = pixel.0.to_be_bytes();
        buf[offset] = bytes[0];
        buf[offset + 1] = bytes[1];
    }

    fn flush(&mut self, data: &[u8]) -> Result<()> {
        let expected = self.buffer_size();
        if data.len() != expected {
            return Err(Error::InvalidBufferSize {
                expected,
                got: data.len(),
            });
        }

        let width = self.inner.width() as u16;
        let height = self.inner.height() as u16;

        // Interpret raw bytes as big-endian RGB565 u16 pixels
        let pixels = data
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]));

        self.inner
            .draw_raw_iter(0, 0, width - 1, height - 1, pixels)?;
        Ok(())
    }
}
