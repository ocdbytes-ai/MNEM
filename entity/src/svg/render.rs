use std::path::Path;

use tiny_skia::Pixmap;
use types::{Mono, Pixel, Rgb565};

use super::loader::SvgData;
use super::{Error, Result};

/// RGB color for rendering.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
}

/// Controls how pixels are colorized for the display.
#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    pub fg: Color,
    pub bg: Color,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            fg: Color::WHITE,
            bg: Color::BLACK,
        }
    }
}

/// Convert an RGBA pixmap to RGB565 display pixels.
/// Uses alpha as a mask to blend foreground over background.
pub fn pixmap_to_rgb565(pixmap: &Pixmap, config: &RenderConfig) -> Vec<Pixel<Rgb565>> {
    let w = pixmap.width();
    let h = pixmap.height();
    let data = pixmap.data();

    (0..h)
        .flat_map(|y| {
            (0..w).map(move |x| {
                let i = ((y * w + x) * 4) as usize;
                let a = data[i + 3] as u16;
                let inv = 255 - a;

                let r = ((config.fg.r as u16 * a + config.bg.r as u16 * inv) / 255) as u8;
                let g = ((config.fg.g as u16 * a + config.bg.g as u16 * inv) / 255) as u8;
                let b = ((config.fg.b as u16 * a + config.bg.b as u16 * inv) / 255) as u8;

                Pixel {
                    x: x as usize,
                    y: y as usize,
                    color: Rgb565::from_rgb(r, g, b),
                }
            })
        })
        .collect()
}

/// Save a pixmap as PNG for debugging.
pub fn save_debug_png(pixmap: &Pixmap, path: impl AsRef<Path>) -> Result<()> {
    pixmap.save_png(path).map_err(|_| Error::Render)
}

impl SvgData {
    /// Rasterize SVG to an RGBA pixel buffer, scaled to fit within
    /// `width x height` and centered.
    pub fn rasterize(&self, width: u32, height: u32) -> Result<Pixmap> {
        let mut pixmap = Pixmap::new(width, height).ok_or(Error::Render)?;

        let (svg_w, svg_h) = self.size();
        let scale_x = width as f32 / svg_w;
        let scale_y = height as f32 / svg_h;
        let scale = scale_x.min(scale_y);

        let offset_x = (width as f32 - svg_w * scale) / 2.0;
        let offset_y = (height as f32 - svg_h * scale) / 2.0;

        let transform =
            tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);

        resvg::render(self.tree(), transform, &mut pixmap.as_mut());

        Ok(pixmap)
    }

    /// Convenience: rasterize + convert to RGB565 in one call.
    pub fn render_rgb565(
        &self,
        width: u32,
        height: u32,
        config: &RenderConfig,
    ) -> Result<Vec<Pixel<Rgb565>>> {
        let pixmap = self.rasterize(width, height)?;
        Ok(pixmap_to_rgb565(&pixmap, config))
    }

    /// Convenience: rasterize + convert to monochrome in one call.
    pub fn render_mono(&self, width: u32, height: u32) -> Result<Vec<Pixel<Mono>>> {
        let pixmap = self.rasterize(width, height)?;
        let w = pixmap.width();
        let h = pixmap.height();
        let data = pixmap.data();

        let pixels = (0..h)
            .flat_map(|y| {
                (0..w).map(move |x| {
                    let i = ((y * w + x) * 4) as usize;
                    Pixel {
                        x: x as usize,
                        y: y as usize,
                        color: Mono(data[i + 3] > 127),
                    }
                })
            })
            .collect();

        Ok(pixels)
    }
}
