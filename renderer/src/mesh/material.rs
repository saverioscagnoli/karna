use crate::Color;
use common::utils::Label;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub texture: Option<TextureKind>,
    pub color: Option<Color>,
}

/// Defines how a texture should be rendered
#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    /// Render the full texture
    Full(Label),
    /// Render a subregion of the texture
    Partial(Label, TextureRegion),
}

/// Defines a rectangular region within a texture in pixel coordinates
#[derive(Debug, Clone, Copy)]
pub struct TextureRegion {
    /// X coordinate of the top-left corner in pixels
    pub x: u32,
    /// Y coordinate of the top-left corner in pixels
    pub y: u32,
    /// Width of the region in pixels
    pub width: u32,
    /// Height of the region in pixels
    pub height: u32,
}

impl TextureRegion {
    /// Create a new texture region
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}
