mod ili9341;
mod ssd1306;

use std::thread::sleep;
use std::time::Duration;

pub use self::ili9341::ILI9341Display;
pub use ssd1306::SSD1306;

// --- Error ---

#[derive(Debug)]
pub enum Error {
    I2c(rppal::i2c::Error),
    Spi(rppal::spi::Error),
    Gpio(rppal::gpio::Error),
    Display(display_interface::DisplayError),
    InvalidBufferSize { expected: usize, got: usize },
    Image(image::ImageError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::I2c(e) => write!(f, "I2C error: {e}"),
            Error::Spi(e) => write!(f, "SPI error: {e}"),
            Error::Gpio(e) => write!(f, "GPIO error: {e}"),
            Error::Display(e) => write!(f, "Display error: {e:?}"),
            Error::InvalidBufferSize { expected, got } => {
                write!(
                    f,
                    "invalid buffer size: expected {expected} bytes, got {got}"
                )
            }
            Error::Image(e) => write!(f, "Image error: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::I2c(e) => Some(e),
            Error::Spi(e) => Some(e),
            Error::Gpio(e) => Some(e),
            Error::Display(_) => None,
            Error::InvalidBufferSize { .. } => None,
            Error::Image(e) => Some(e),
        }
    }
}

impl From<rppal::i2c::Error> for Error {
    fn from(e: rppal::i2c::Error) -> Self {
        Error::I2c(e)
    }
}

impl From<rppal::spi::Error> for Error {
    fn from(e: rppal::spi::Error) -> Self {
        Error::Spi(e)
    }
}

impl From<rppal::gpio::Error> for Error {
    fn from(e: rppal::gpio::Error) -> Self {
        Error::Gpio(e)
    }
}

impl From<image::ImageError> for Error {
    fn from(e: image::ImageError) -> Self {
        Error::Image(e)
    }
}

impl From<display_interface::DisplayError> for Error {
    fn from(e: display_interface::DisplayError) -> Self {
        Error::Display(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

// --- Pixel formats (re-exported from types crate) ---

pub use types::{Mono, Pixel, PixelFormat, Rgb565};

// --- Display trait ---

pub trait DisplayInterface {
    type Pixel: PixelFormat;

    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn encode_pixel(&self, pixel: &Self::Pixel, buf: &mut [u8], x: usize, y: usize);
    fn flush(&mut self, data: &[u8]) -> Result<()>;

    fn buffer_size(&self) -> usize {
        self.width() * self.height() * Self::Pixel::BITS_PER_PIXEL / 8
    }

    fn fill_screen(&mut self, pixel: &Self::Pixel) -> Result<()> {
        let mut buf = vec![0u8; self.buffer_size()];
        for y in 0..self.height() {
            for x in 0..self.width() {
                self.encode_pixel(pixel, &mut buf, x, y);
            }
        }
        self.flush(&buf)
    }

    fn draw(&mut self, pixels: &[Pixel<Self::Pixel>]) -> Result<()> {
        let mut buf = vec![0u8; self.buffer_size()];
        for pixel in pixels {
            self.encode_pixel(&pixel.color, &mut buf, pixel.x, pixel.y);
        }
        self.flush(&buf)
    }

    fn test_display(&mut self) -> Result<()> {
        self.fill_screen(&Self::Pixel::WHITE)?;
        sleep(Duration::from_secs(2));
        self.fill_screen(&Self::Pixel::BLACK)?;
        Ok(())
    }
}
