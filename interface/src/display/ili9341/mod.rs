mod cmd;

use display_interface::WriteOnlyDataCommand;
use embedded_hal::digital::OutputPin;

pub struct ILI9341Display<IFACE, RESET> {
    inner: ili9341::Ili9341<IFACE, RESET>,
}

impl<IFACE, RESET> ILI9341Display<IFACE, RESET>
where
    IFACE: WriteOnlyDataCommand,
    RESET: OutputPin,
{
    pub fn new(inner: ili9341::Ili9341<IFACE, RESET>) -> Self {
        Self { inner }
    }
}
