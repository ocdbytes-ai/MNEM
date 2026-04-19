use std::env;

use entity::effect::DisplacementParams;
use entity::svg::{RenderConfig, SvgData, pixmap_to_rgb565, save_debug_png};

#[cfg(target_os = "linux")]
use interface::display::{DisplayInterface, ILI9341Display, Rgb565};
#[cfg(target_os = "linux")]
use std::thread::sleep;
#[cfg(target_os = "linux")]
use std::time::Duration;

#[cfg(target_os = "linux")]
fn run_ssd1306() -> Result<(), Box<dyn std::error::Error>> {
    use interface::display::SSD1306;
    let i2c = rppal::i2c::I2c::new()?;
    let mut display = SSD1306::new(i2c)?;
    display.test_display()?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn run_ili9341() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = ILI9341Display::setup()?;
    display.test_display()?;
    Ok(())
}

#[cfg(target_os = "linux")]
const IMAGE: &[u8] = include_bytes!("../../interface/assets/image.jpg");

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn project_rgb() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = ILI9341Display::setup()?;
    project(&mut display)
}

const ENTITY: &[u8] = include_bytes!("../assets/entity.svg");

fn debug_entity() -> Result<(), Box<dyn std::error::Error>> {
    let svg = SvgData::from_bytes_no_filters(ENTITY)?;
    eprintln!("SVG size: {:?}", svg.size());

    // Step 1: rasterize the plain circle
    let source = svg.rasterize(240, 320)?;
    save_debug_png(&source, "debug_1_circle.png")?;
    eprintln!("Saved debug_1_circle.png (raw circle, no filter)");

    // Step 2: apply noise displacement
    let params = DisplacementParams::default();
    let displaced = params.apply(&source);
    save_debug_png(&displaced, "debug_2_displaced.png")?;
    eprintln!("Saved debug_2_displaced.png (after displacement)");

    // Step 3: apply color conversion (white on black)
    let config = RenderConfig::default();
    let _pixels = pixmap_to_rgb565(&displaced, &config);
    eprintln!("Converted to {} RGB565 pixels", _pixels.len());

    eprintln!("\nPipeline: SVG → rasterize → displace → colorize → display");
    Ok(())
}

#[cfg(target_os = "linux")]
const FRAME_COUNT: usize = 60;

#[cfg(target_os = "linux")]
fn render_entity() -> Result<(), Box<dyn std::error::Error>> {
    let mut display = ILI9341Display::setup()?;
    let svg = SvgData::from_bytes_no_filters(ENTITY)?;
    let w = display.width() as u32;
    let h = display.height() as u32;
    let source = svg.rasterize(w, h)?;
    let config = RenderConfig::default();

    // Pre-render all frames at startup
    eprintln!("Pre-rendering {FRAME_COUNT} frames...");
    let frames: Vec<_> = (0..FRAME_COUNT)
        .map(|i| {
            let t = i as f64 / FRAME_COUNT as f64;
            let breathe = 1.0 + 0.15 * (t * std::f64::consts::TAU).sin();
            let params = DisplacementParams {
                time: i as f64 * 0.5,
                scale: 60.0 * breathe as f32,
                ..DisplacementParams::default()
            };
            let displaced = params.apply(&source);
            pixmap_to_rgb565(&displaced, &config)
        })
        .collect();
    eprintln!("Done. Playing back.");

    // Smooth playback — zero per-frame computation
    loop {
        for frame in &frames {
            display.draw(frame)?;
            sleep(Duration::from_millis(33));
        }
    }
}

fn main() {
    let cmd = env::args().nth(1);
    let cmd = cmd.as_deref().unwrap_or("debug");

    let result = match cmd {
        #[cfg(target_os = "linux")]
        "test-ssd1306" => run_ssd1306(),
        #[cfg(target_os = "linux")]
        "test-ili9341" => run_ili9341(),
        #[cfg(target_os = "linux")]
        "project" => project_rgb(),
        #[cfg(target_os = "linux")]
        "entity" => render_entity(),
        "debug" => debug_entity(),
        other => {
            eprintln!("Unknown command: {other}");
            #[cfg(target_os = "linux")]
            eprintln!("Usage: entity [test-ssd1306|test-ili9341|project|entity|debug]");
            #[cfg(not(target_os = "linux"))]
            eprintln!("Usage: entity [debug]");
            std::process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
