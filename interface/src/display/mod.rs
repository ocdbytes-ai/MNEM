mod ili9341;
mod ssd1306;

pub use self::ili9341::ILI9341Display;
pub use ssd1306::SSD1306;

#[derive(Debug)]
pub enum Error {
    I2c(rppal::i2c::Error),
    Spi(rppal::spi::Error),
    Gpio(rppal::gpio::Error),
    Display(display_interface::DisplayError),
    InvalidBufferSize { expected: usize, got: usize },
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

impl From<display_interface::DisplayError> for Error {
    fn from(e: display_interface::DisplayError) -> Self {
        Error::Display(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait DisplayInterface {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn buffer_size(&self) -> usize;
    fn flush(&mut self, data: &[u8]) -> Result<()>;
}
