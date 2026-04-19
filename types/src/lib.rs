pub trait PixelFormat {
    const BITS_PER_PIXEL: usize;
    const WHITE: Self;
    const BLACK: Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mono(pub bool);

impl PixelFormat for Mono {
    const BITS_PER_PIXEL: usize = 1;
    const WHITE: Self = Self(true);
    const BLACK: Self = Self(false);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb565(pub u16);

impl Rgb565 {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let r5 = (r as u16 >> 3) & 0x1F;
        let g6 = (g as u16 >> 2) & 0x3F;
        let b5 = (b as u16 >> 3) & 0x1F;
        Self((r5 << 11) | (g6 << 5) | b5)
    }
}

impl PixelFormat for Rgb565 {
    const BITS_PER_PIXEL: usize = 16;
    const WHITE: Self = Self(0xFFFF);
    const BLACK: Self = Self(0x0000);
}

pub struct Pixel<P: PixelFormat> {
    pub x: usize,
    pub y: usize,
    pub color: P,
}
