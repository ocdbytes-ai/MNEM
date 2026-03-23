use std::env;
use std::thread::sleep;
use std::time::Duration;

use interface::display::{DisplayInterface, ILI9341Display, Mono, Rgb565};

fn run_ssd1306() -> Result<(), Box<dyn std::error::Error>> {
    use interface::display::SSD1306;
    let i2c = rppal::i2c::I2c::new()?;
    let mut display = SSD1306::new(i2c)?;
    display.fill_screen(&Mono(true))?;
    sleep(Duration::from_secs(2));
    display.fill_screen(&Mono(false))?;
    Ok(())
}

fn run_ili9341() -> Result<(), Box<dyn std::error::Error>> {
    use interface::display::ILI9341Display;
    let mut display = ILI9341Display::setup()?;
    display.fill_screen(&Rgb565::WHITE)?;
    sleep(Duration::from_secs(2));
    display.fill_screen(&Rgb565::BLACK)?;
    Ok(())
}

const IMAGE: &[u8] = include_bytes!("../assets/image.jpg");

fn project<D: DisplayInterface<Pixel = Rgb565>>(
    display: &mut D,
) -> Result<(), Box<dyn std::error::Error>> {
    let pixels = interface::image::load_rgb565_bytes(
        IMAGE,
        display.width() as u32,
        display.height() as u32,
    )?;
    display.draw(&pixels)?;
    sleep(Duration::from_secs(10));
    Ok(())
}

fn project_rgb() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = ILI9341Display::setup()?;
    project(&mut display)
}

fn main() {
    let display_type = env::args().nth(1);
    let display_type = display_type.as_deref().unwrap_or("ssd1306");

    let result = match display_type {
        "ssd1306" => run_ssd1306(),
        "ili9341" => run_ili9341(),
        "project" => project_rgb(),
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
