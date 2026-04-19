mod loader;
mod render;

pub use loader::SvgData;
pub use render::{Color, RenderConfig, pixmap_to_rgb565, save_debug_png};

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parse(resvg::usvg::Error),
    Render,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "SVG I/O error: {e}"),
            Error::Parse(e) => write!(f, "SVG parse error: {e}"),
            Error::Render => write!(f, "SVG render failed (invalid dimensions?)"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            Error::Parse(e) => Some(e),
            Error::Render => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<resvg::usvg::Error> for Error {
    fn from(e: resvg::usvg::Error) -> Self {
        Error::Parse(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
