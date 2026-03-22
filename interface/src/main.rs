use std::env;
use std::thread::sleep;
use std::time::Duration;

use interface::display::{DisplayInterface, ILI9341Display, SSD1306};

fn run_ssd1306() -> interface::display::Result<()> {
    let i2c = rppal::i2c::I2c::new()?;
    let mut display = SSD1306::new(i2c)?;

    display.all_on()?;
    sleep(Duration::from_secs(2));
    display.all_off()?;

    Ok(())
}

fn run_ili9341() -> interface::display::Result<()> {
    use display_interface_spi::SPIInterface;
    use ili9341::{DisplaySize240x320, Orientation};
    use interface::hal;
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
    .map_err(interface::display::Error::Display)?;

    let mut display = ILI9341Display::new(ili);

    // Fill screen white
    let white = 0xFFFFu16;
    let framebuffer: Vec<u8> = std::iter::repeat_n(white.to_be_bytes(), 240 * 320)
        .flatten()
        .collect();
    display.flush(&framebuffer)?;

    sleep(Duration::from_secs(2));

    // Fill screen black
    let framebuffer = vec![0u8; 240 * 320 * 2];
    display.flush(&framebuffer)?;

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let display_type = args.get(1).map(String::as_str).unwrap_or("ssd1306");

    let result = match display_type {
        "ssd1306" => run_ssd1306(),
        "ili9341" => run_ili9341(),
        other => {
            eprintln!("Unknown display type: {other}");
            eprintln!("Usage: interface [ssd1306|ili9341]");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
