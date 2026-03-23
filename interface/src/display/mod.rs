mod ili9341;
mod ssd1306;

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

// --- Pixel formats ---

pub trait PixelFormat {
    const BITS_PER_PIXEL: usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mono(pub bool);

impl PixelFormat for Mono {
    const BITS_PER_PIXEL: usize = 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb565(pub u16);

impl Rgb565 {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let r5 = (r as u16 >> 3) & 0x1F;
        let g6 = (g as u16 >> 2) & 0x3F;
        let b5 = (b as u16 >> 3) & 0x1F;
        Self((r5 << 11) | (g6 << 5) | b5)
    }

    pub const WHITE: Self = Self(0xFFFF);
    pub const BLACK: Self = Self(0x0000);
}

impl PixelFormat for Rgb565 {
    const BITS_PER_PIXEL: usize = 16;
}

pub struct Pixel<P: PixelFormat> {
    pub x: usize,
    pub y: usize,
    pub color: P,
}

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
}
