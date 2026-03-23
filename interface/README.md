# Interface

Rust library for display drivers and hardware abstraction on Raspberry Pi.

## Architecture

```
src/
├── lib.rs                  # Crate root
├── main.rs                 # CLI entry point
├── hal.rs                  # rppal -> embedded-hal adapters (SPI, GPIO, Delay)
├── image.rs                # Image loading and pixel format conversion
└── display/
    ├── mod.rs              # DisplayInterface trait, error types, pixel formats
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

## Features

### Display Abstraction

A trait-based display interface that works across different display types:

- `fill_screen` -- fill the entire display with a single color
- `draw` -- render a set of positioned pixels
- `encode_pixel` -- display-specific pixel encoding (page-based for SSD1306, linear for ILI9341)
- `buffer_size` -- automatically computed from display dimensions and pixel format

### Pixel Formats

- **`Mono(bool)`** -- 1-bit monochrome for OLED displays
- **`Rgb565(u16)`** -- 16-bit color with `from_rgb(r, g, b)` constructor and `WHITE`/`BLACK` constants

### Image Loading

Load and convert images for display rendering (PNG, JPEG, BMP):

- `load_rgb565(path, width, height)` -- load from file to RGB565 pixels
- `load_rgb565_bytes(bytes, width, height)` -- load from embedded bytes (via `include_bytes!`)
- `load_mono(path, width, height)` -- load from file to monochrome pixels
- `load_mono_bytes(bytes, width, height)` -- load from embedded bytes

Images are automatically resized to the target display dimensions.

### HAL Adapters

Bridges between `rppal` (Raspberry Pi) and `embedded-hal` 1.0 traits:

- `SpiDevice` -- rppal SPI to `embedded_hal::spi::SpiDevice`
- `OutputPin` -- rppal GPIO to `embedded_hal::digital::OutputPin`
- `Delay` -- `std::thread::sleep` based `embedded_hal::delay::DelayNs`

## Build

### Prerequisites

```sh
# Install cross-compilation toolchain (macOS)
# for 64 bit
rustup target add aarch64-unknown-linux-gnu
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu
# 32 bit - armv7
rustup target add arm-unknown-linux-gnueabihf
brew install messense/macos-cross-toolchains/armv7-unknown-linux-gnueabihf
# 32 bit - armv6
rustup target add armv7-unknown-linux-gnueabihf
brew install messense/macos-cross-toolchains/arm-unknown-linux-gnueabihf
```

Ensure `.cargo/config.toml` exists with:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-unknown-linux-gnu-gcc"

[target.armv7-unknown-linux-gnueabihf]
linker = "armv7-unknown-linux-gnueabihf-gcc"

[target.arm-unknown-linux-gnueabihf]
linker = "arm-unknown-linux-gnueabihf-gcc"
```

### Compile

```sh
# for 64 bit
cargo build --release --target aarch64-unknown-linux-gnu
# 32 bit - armv7
cargo build --release --target armv7-unknown-linux-gnueabihf
# 32 bit - armv6
cargo build --release --target arm-unknown-linux-gnueabihf 
```

## Run

Copy the binary to your Raspberry Pi and run:

```sh
# SSD1306 OLED test (default)
./interface ssd1306

# ILI9341 TFT test
./interface ili9341

# Display an image on ILI9341
./interface project
```

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
use interface::image;
use std::path::Path;

// Render an image
let pixels = image::load_rgb565(Path::new("assets/photo.png"), 320, 240)?;
display.draw(&pixels)?;
```
