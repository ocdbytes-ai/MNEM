# Entity

The creature itself -- SVG loading, noise displacement, and the main binary entry point.

## Architecture

```
src/
├── lib.rs              # Crate root
├── main.rs             # CLI entry point (debug on macOS, hardware on Linux)
├── svg/
│   ├── mod.rs          # Error type, Result alias, re-exports
│   ├── loader.rs       # SvgData: parse SVG from file or bytes
│   └── render.rs       # Color, RenderConfig, pixmap-to-pixel conversion
└── effect/
    ├── mod.rs          # Re-exports
    └── displacement.rs # DisplacementParams: Perlin noise displacement

assets/
└── entity.svg          # Creature SVG (circle + filter definition)
```

## Rendering Pipeline

The creature's visual form starts as a circle in `entity.svg`. Since resvg doesn't
support `feDisplacementMap`, the noise displacement is implemented in Rust, giving
full control over animation parameters.

```
SVG (circle) → rasterize (resvg) → displace (Perlin noise) → colorize (fg/bg) → display
```

1. **Load** -- parse SVG with filters stripped (`from_bytes_no_filters`)
2. **Rasterize** -- render to RGBA pixmap via resvg, scaled and centered
3. **Displace** -- apply fractal noise displacement to warp the circle into an organic blob
4. **Colorize** -- alpha-blend foreground over background (white on black by default)
5. **Convert** -- output as `Vec<Pixel<Rgb565>>` or `Vec<Pixel<Mono>>`

## Displacement Effect

`DisplacementParams` controls the noise displacement:

| Parameter | Default | Effect |
|-----------|---------|--------|
| `frequency` | 0.0667 | Noise zoom level. Lower = larger blob features |
| `octaves` | 3 | Detail layers. More = more textured edges |
| `scale` | 60.0 | Max pixel displacement. Higher = more warped |
| `seed` | 6496 | Shape determinism. Different seed = different shape |
| `time` | 0.0 | Z-axis offset for smooth animation |

### Animation

Increment `time` per frame for organic drifting:

```rust
use entity::effect::DisplacementParams;

for frame in 0.. {
    let params = DisplacementParams {
        time: frame as f64 * 0.05,
        ..DisplacementParams::default()
    };
    let displaced = params.apply(&source);
    let pixels = pixmap_to_rgb565(&displaced, &config);
    display.draw(&pixels)?;
}
```

Change `seed` for a completely different shape. Change `scale` for breathing effects.

## Usage

```rust
use entity::svg::{SvgData, RenderConfig, pixmap_to_rgb565};
use entity::effect::DisplacementParams;

// Load SVG (strip filters, effects handled in Rust)
let svg = SvgData::from_bytes_no_filters(include_bytes!("../assets/entity.svg"))?;

// Rasterize the plain circle
let source = svg.rasterize(240, 320)?;

// Apply noise displacement
let params = DisplacementParams::default();
let displaced = params.apply(&source);

// Convert to display pixels (white on black)
let config = RenderConfig::default();
let pixels = pixmap_to_rgb565(&displaced, &config);
display.draw(&pixels)?;
```

### Custom Colors

```rust
use entity::svg::{Color, RenderConfig};

let config = RenderConfig {
    fg: Color { r: 0, g: 255, b: 200 },  // teal
    bg: Color::BLACK,
};
```

## CLI

```sh
# Debug (works on macOS, no hardware needed)
./entity debug       # Saves debug_1_circle.png + debug_2_displaced.png

# Hardware (Linux/RPi only)
./entity entity          # Render creature on ILI9341
./entity test-ssd1306    # Test SSD1306 OLED
./entity test-ili9341    # Test ILI9341 TFT
./entity project         # Display an image on ILI9341
```
