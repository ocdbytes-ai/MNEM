use noise::{Fbm, MultiFractal, NoiseFn, Perlin};
use tiny_skia::Pixmap;

/// Parameters controlling the noise displacement effect.
///
/// Matches the SVG filter chain: feTurbulence → feDisplacementMap.
///
/// - `frequency`: how "zoomed in" the noise is. Lower = larger blobs.
/// - `octaves`: layers of detail. 1 = smooth, 3+ = more textured.
/// - `scale`: max pixel displacement. Higher = more warped.
/// - `seed`: deterministic randomness. Different seed = different shape.
/// - `time`: z-axis offset for smooth animation. Increment per frame.
#[derive(Debug, Clone, Copy)]
pub struct DisplacementParams {
    pub scale: f32,
    pub frequency: f64,
    pub octaves: usize,
    pub seed: u32,
    pub time: f64,
}

impl Default for DisplacementParams {
    fn default() -> Self {
        Self {
            scale: 60.0,
            frequency: 0.0667,
            octaves: 3,
            seed: 6496,
            time: 0.0,
        }
    }
}

impl DisplacementParams {
    /// Apply noise displacement to a source pixmap.
    ///
    /// For each output pixel (x, y):
    ///   1. Sample fractal noise to get displacement (dx, dy)
    ///   2. Copy the source pixel at (x + dx, y + dy)
    ///   3. Out-of-bounds samples produce transparent pixels
    pub fn apply(&self, source: &Pixmap) -> Pixmap {
        let w = source.width();
        let h = source.height();
        let mut output = Pixmap::new(w, h).expect("same dimensions as source");
        let src = source.data();
        let dst = output.data_mut();

        // Two independent noise fields for x and y displacement,
        // like SVG's xChannelSelector="R" yChannelSelector="G"
        let noise_x = Fbm::<Perlin>::new(self.seed)
            .set_frequency(self.frequency)
            .set_octaves(self.octaves);

        let noise_y = Fbm::<Perlin>::new(self.seed.wrapping_add(1))
            .set_frequency(self.frequency)
            .set_octaves(self.octaves);

        let half_scale = self.scale * 0.5;

        for y in 0..h {
            for x in 0..w {
                // Sample noise at this pixel. Returns ~[-1, 1].
                // The time dimension gives smooth animation.
                let nx = noise_x.get([x as f64, y as f64, self.time]);
                let ny = noise_y.get([x as f64, y as f64, self.time]);

                // Displace: noise * scale/2 gives range [-scale/2, +scale/2]
                let sx = (x as f32 + nx as f32 * half_scale).round() as i32;
                let sy = (y as f32 + ny as f32 * half_scale).round() as i32;

                // Bounds check — out of bounds stays transparent (0,0,0,0)
                if sx >= 0 && sx < w as i32 && sy >= 0 && sy < h as i32 {
                    let src_i = (sy as usize * w as usize + sx as usize) * 4;
                    let dst_i = (y as usize * w as usize + x as usize) * 4;
                    dst[dst_i..dst_i + 4].copy_from_slice(&src[src_i..src_i + 4]);
                }
            }
        }

        output
    }
}
