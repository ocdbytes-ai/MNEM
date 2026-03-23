mod cmd;

use crate::display::Result;
use crate::hal;
use display_interface::WriteOnlyDataCommand;
use display_interface_spi::SPIInterface;
use embedded_hal::digital::OutputPin;

pub struct ILI9341Display<IFACE, RESET> {
    inner: ili9341::Ili9341<IFACE, RESET>,
}

impl<IFACE, RESET> ILI9341Display<IFACE, RESET>
where
    IFACE: WriteOnlyDataCommand,
    RESET: OutputPin,
{
    #[must_use]
    pub fn new(inner: ili9341::Ili9341<IFACE, RESET>) -> Self {
        Self { inner }
    }
}

impl ILI9341Display<SPIInterface<hal::SpiDevice, hal::OutputPin>, hal::OutputPin> {
    pub fn setup() -> Result<Self> {
        use ili9341::{DisplaySize240x320, Orientation};
        use rppal::gpio::Gpio;
        use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

        // SPI bus at 16 MHz, Mode 0 (CPOL=0, CPHA=0)
        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0)?;
        let spi_device = hal::SpiDevice::new(spi);

        let gpio = Gpio::new()?;
        let dc = hal::OutputPin::new(gpio.get(25)?.into_output()); // DC pin (GPIO 25)
        let reset = hal::OutputPin::new(gpio.get(27)?.into_output()); // RST pin (GPIO 27)

        let spi_iface = SPIInterface::new(spi_device, dc);
        let mut delay = hal::Delay;

        let ili = ili9341::Ili9341::new(
            spi_iface,
            reset,
            &mut delay,
            Orientation::Landscape,
            DisplaySize240x320,
        )
        .map_err(crate::display::Error::Display)?;

        Ok(Self::new(ili))
    }
}
