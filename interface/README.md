# Interface

Rust library for display drivers and hardware abstraction on Raspberry Pi.

Pixel types (`Rgb565`, `Mono`, `Pixel`, `PixelFormat`) live in the `types` crate
and are re-exported here for convenience.

## Architecture

```
src/
├── lib.rs                  # Crate root
├── hal.rs                  # rppal -> embedded-hal adapters (SPI, GPIO, Delay)
├── image.rs                # Image loading and pixel format conversion
└── display/
    ├── mod.rs              # DisplayInterface trait, error types, re-exports from types
    ├── ssd1306/            # SSD1306 OLED driver (I2C, 128x64, monochrome)
    │   ├── mod.rs
    │   ├── cmd.rs
    │   └── constants.rs
    └── ili9341/            # ILI9341 TFT driver (SPI, 240x320, RGB565)
        ├── mod.rs
        └── cmd.rs
```

## Supported Displays

| Display | Interface | Resolution | Pixel Format | Connection |
|---------|-----------|------------|--------------|------------|
| SSD1306 | I2C | 128x64 | Monochrome (1-bit) | `I2C_ADDR = 0x3C` |
| ILI9341 | SPI | 240x320 | RGB565 (16-bit) | SPI0, GPIO 25 (DC), GPIO 27 (RST) |

## Display Abstraction

A trait-based display interface (`DisplayInterface`) that works across different display types:

- `fill_screen` -- fill the entire display with a single color
- `draw` -- render a set of positioned pixels
- `test_display` -- flash white/black to verify the display works
- `buffer_size` -- automatically computed from display dimensions and pixel format

## Pixel Formats

Defined in the `types` crate, re-exported as `interface::display::{Rgb565, Mono, Pixel, PixelFormat}`.

Both formats implement `PixelFormat` with `WHITE` and `BLACK` constants.

- **`Mono(bool)`** -- 1-bit monochrome for OLED displays
- **`Rgb565(u16)`** -- 16-bit color with `from_rgb(r, g, b)` constructor

## Image Loading

Load and convert raster images (PNG, JPEG, BMP) for display rendering:

- `load_rgb565(path, width, height)` -- load from file to RGB565 pixels
- `load_rgb565_bytes(bytes, width, height)` -- load from embedded bytes (`include_bytes!`)
- `load_mono(path, width, height)` -- load from file to monochrome pixels
- `load_mono_bytes(bytes, width, height)` -- load from embedded bytes

Images are resized to the target display dimensions using Lanczos3 filtering.

## HAL Adapters

Bridges between `rppal` (Raspberry Pi) and `embedded-hal` 1.0 traits:

- `SpiDevice` -- rppal SPI to `embedded_hal::spi::SpiDevice`
- `OutputPin` -- rppal GPIO to `embedded_hal::digital::OutputPin`
- `Delay` -- `std::thread::sleep` based `embedded_hal::delay::DelayNs`

## Usage

```rust
use interface::display::{DisplayInterface, SSD1306, Mono};

// SSD1306 over I2C
let i2c = rppal::i2c::I2c::new()?;
let mut display = SSD1306::new(i2c)?;
display.fill_screen(&Mono(true))?;
```

```rust
use interface::display::{DisplayInterface, ILI9341Display, Rgb565};

// ILI9341 over SPI
let mut display = ILI9341Display::setup()?;
display.fill_screen(&Rgb565::from_rgb(255, 0, 0))?;
```

```rust
// Render an image
let pixels = interface::image::load_rgb565(Path::new("photo.png"), 320, 240)?;
display.draw(&pixels)?;
```
