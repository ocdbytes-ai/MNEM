use rppal::gpio;
use rppal::spi;

// --- SPI adapter ---

pub struct SpiDevice {
    spi: spi::Spi,
}

impl SpiDevice {
    pub fn new(spi: spi::Spi) -> Self {
        Self { spi }
    }
}

#[derive(Debug)]
pub struct SpiError(spi::Error);

impl embedded_hal::spi::Error for SpiError {
    fn kind(&self) -> embedded_hal::spi::ErrorKind {
        embedded_hal::spi::ErrorKind::Other
    }
}

impl std::fmt::Display for SpiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl embedded_hal::spi::ErrorType for SpiDevice {
    type Error = SpiError;
}

impl embedded_hal::spi::SpiDevice for SpiDevice {
    fn transaction(
        &mut self,
        operations: &mut [embedded_hal::spi::Operation<'_, u8>],
    ) -> Result<(), Self::Error> {
        for op in operations {
            match op {
                embedded_hal::spi::Operation::Read(buf) => {
                    self.spi.read(buf).map_err(SpiError)?;
                }
                embedded_hal::spi::Operation::Write(buf) => {
                    self.spi.write(buf).map_err(SpiError)?;
                }
                embedded_hal::spi::Operation::Transfer(read, write) => {
                    self.spi.transfer(read, write).map_err(SpiError)?;
                }
                embedded_hal::spi::Operation::TransferInPlace(buf) => {
                    let write_copy = buf.to_vec();
                    self.spi.transfer(buf, &write_copy).map_err(SpiError)?;
                }
                embedded_hal::spi::Operation::DelayNs(ns) => {
                    std::thread::sleep(std::time::Duration::from_nanos(*ns as u64));
                }
            }
        }
        Ok(())
    }
}

// --- GPIO OutputPin adapter ---

pub struct OutputPin {
    pin: gpio::OutputPin,
}

impl OutputPin {
    pub fn new(pin: gpio::OutputPin) -> Self {
        Self { pin }
    }
}

impl embedded_hal::digital::ErrorType for OutputPin {
    type Error = core::convert::Infallible;
}

impl embedded_hal::digital::OutputPin for OutputPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high();
        Ok(())
    }
}

// --- Delay adapter ---

pub struct Delay;

impl embedded_hal::delay::DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        std::thread::sleep(std::time::Duration::from_nanos(ns as u64));
    }
}
