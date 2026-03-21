mod ssd1306;

pub type Result<T> = std::result::Result<T, rppal::i2c::Error>;

pub use ssd1306::SSD1306;

pub trait DisplayInterface {
    fn init(&mut self) -> Result<()>;
    fn flush(&mut self, data: &[u8]) -> Result<()>;
}
