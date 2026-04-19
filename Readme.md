# Mnem

Entity that lives, remembers and breathes.

A bioluminescent alien creature (Voreln) that lives on a Raspberry Pi, expresses emotion
through form and light, is fed via Telegram, and gradually learns its own behavioral
personality through a custom ML model trained over three months.

See [Plan.md](Plan.md) for the full project plan.

## Project Structure

```
project-mnem/
├── types/              Shared pixel types (platform-independent)
├── interface/          Display drivers, HAL adapters, image loading (Linux/RPi)
├── entity/             Creature SVG + noise displacement, CLI entry point
├── .cargo/config.toml  Cross-compilation linker config (aarch64, armv7, arm)
└── Plan.md             Full project plan (phases, ML track, architecture)
```

| Crate | Type | Description |
|-------|------|-------------|
| `types` | lib | `PixelFormat`, `Pixel`, `Rgb565`, `Mono` -- shared across crates, no hardware deps |
| `interface` | lib | Display trait, SSD1306/ILI9341 drivers, HAL adapters, image loading |
| `entity` | lib + bin | SVG loader, noise displacement effect, CLI entry point |

### Dependency Graph

```
types (pure data, compiles everywhere)
  ^            ^
  |            |
interface    entity lib
(rppal,      (resvg, tiny-skia, noise)
 ili9341)        ^
  ^              |
  |     entity bin (cfg-gated on Linux)
  +---------|
```

`entity` lib compiles on macOS for development and debugging.
Hardware commands in the binary are gated behind `#[cfg(target_os = "linux")]`.

## Build

### macOS (debug/development)

```sh
cargo run -p entity -- debug    # renders SVG pipeline to PNG files for inspection
```

### Cross-compile (Raspberry Pi)

```sh
# Prerequisites (one-time setup)
rustup target add aarch64-unknown-linux-gnu
brew install messense/macos-cross-toolchains/aarch64-unknown-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

### Run on Pi

```sh
scp target/aarch64-unknown-linux-gnu/release/entity pi@raspberrypi.local:~/

# On the Pi:
./entity entity          # Render creature on ILI9341
./entity test-ssd1306    # Test SSD1306 OLED
./entity test-ili9341    # Test ILI9341 TFT
./entity project         # Display an image on ILI9341
./entity debug           # Save pipeline PNGs for inspection
```

## Stack

| Layer | Tech |
|-------|------|
| Renderer + state machine | Rust |
| Display drivers | Rust (`rppal`, `embedded-hal`, `ili9341`) |
| SVG rendering | Rust (`resvg`, `tiny-skia`) |
| Noise displacement | Rust (`noise`) |
| Telegram bot | Python (`python-telegram-bot`) |
| Behavior engine | Python (Claude API during collection, local ML model after) |
| Shared state | SQLite |
