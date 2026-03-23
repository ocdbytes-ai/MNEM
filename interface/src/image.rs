use crate::display::{Mono, Pixel, Result, Rgb565};
use image::imageops::FilterType;
use std::path::Path;

/// Load an image from a file, resize to the given dimensions, and convert to RGB565 pixels.
pub fn load_rgb565(path: &Path, width: u32, height: u32) -> Result<Vec<Pixel<Rgb565>>> {
    let img = image::open(path)?
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_rgb8();

    let pixels = img
        .enumerate_pixels()
        .map(|(x, y, rgb)| Pixel {
            x: x as usize,
            y: y as usize,
            color: Rgb565::from_rgb(rgb[0], rgb[1], rgb[2]),
        })
        .collect();

    Ok(pixels)
}

/// Load an image from raw bytes (e.g. `include_bytes!`), resize, and convert to RGB565 pixels.
pub fn load_rgb565_bytes(bytes: &[u8], width: u32, height: u32) -> Result<Vec<Pixel<Rgb565>>> {
    let img = image::load_from_memory(bytes)?
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_rgb8();

    let pixels = img
        .enumerate_pixels()
        .map(|(x, y, rgb)| Pixel {
            x: x as usize,
            y: y as usize,
            color: Rgb565::from_rgb(rgb[0], rgb[1], rgb[2]),
        })
        .collect();

    Ok(pixels)
}

/// Load an image from a file, resize to the given dimensions, and convert to monochrome pixels.
/// Pixels brighter than 127 are on.
pub fn load_mono(path: &Path, width: u32, height: u32) -> Result<Vec<Pixel<Mono>>> {
    let img = image::open(path)?
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_luma8();

    let pixels = img
        .enumerate_pixels()
        .map(|(x, y, luma)| Pixel {
            x: x as usize,
            y: y as usize,
            color: Mono(luma[0] > 127),
        })
        .collect();

    Ok(pixels)
}

/// Load an image from raw bytes, resize, and convert to monochrome pixels.
pub fn load_mono_bytes(bytes: &[u8], width: u32, height: u32) -> Result<Vec<Pixel<Mono>>> {
    let img = image::load_from_memory(bytes)?
        .resize_exact(width, height, FilterType::Lanczos3)
        .to_luma8();

    let pixels = img
        .enumerate_pixels()
        .map(|(x, y, luma)| Pixel {
            x: x as usize,
            y: y as usize,
            color: Mono(luma[0] > 127),
        })
        .collect();

    Ok(pixels)
}
