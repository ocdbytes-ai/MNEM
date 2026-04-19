use std::path::Path;

use resvg::usvg::{Options, Tree};

use super::Result;

/// Parsed SVG data ready for rendering.
pub struct SvgData {
    tree: Tree,
}

impl SvgData {
    /// Parse SVG from raw bytes (e.g. `include_bytes!`).
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let tree = Tree::from_data(data, &Options::default())?;
        Ok(Self { tree })
    }

    /// Parse SVG from raw bytes, stripping all `filter="..."` attributes
    /// so resvg renders the raw shapes without unsupported filter effects.
    /// Use this when applying effects (like displacement) in Rust code.
    pub fn from_bytes_no_filters(data: &[u8]) -> Result<Self> {
        let svg = String::from_utf8_lossy(data);
        let clean = strip_filters(&svg);
        let tree = Tree::from_data(clean.as_bytes(), &Options::default())?;
        Ok(Self { tree })
    }

    /// Load and parse SVG from a file path.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let data = std::fs::read(path)?;
        Self::from_bytes(&data)
    }

    /// Access the underlying usvg tree.
    pub fn tree(&self) -> &Tree {
        &self.tree
    }

    /// Original SVG dimensions (width, height).
    pub fn size(&self) -> (f32, f32) {
        let s = self.tree.size();
        (s.width(), s.height())
    }
}

/// Remove all `filter="..."` attributes from SVG markup.
fn strip_filters(svg: &str) -> String {
    let mut result = svg.to_string();
    while let Some(start) = result.find("filter=\"") {
        let rest = &result[start + 8..];
        if let Some(end) = rest.find('"') {
            // Remove `filter="..."` including the trailing quote
            result.replace_range(start..start + 8 + end + 1, "");
        } else {
            break;
        }
    }
    result
}
