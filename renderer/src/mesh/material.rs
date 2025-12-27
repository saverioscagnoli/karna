use math::{Size, Vector2};
use utils::Label;

use crate::Color;

#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    Full(Label),
    Partial(Label, u32, u32, u32, u32),
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub texture: Option<TextureKind>,
}

impl Default for Material {
    fn default() -> Self {
        Self {
            color: Color::White,
            texture: None,
        }
    }
}

impl Material {
    pub fn new(color: Color, texture: Option<TextureKind>) -> Self {
        Self { color, texture }
    }

    #[inline]
    pub fn new_color(color: Color) -> Self {
        Self {
            color,
            texture: None,
        }
    }

    #[inline]
    pub fn new_texture(texture: TextureKind) -> Self {
        Self {
            color: Color::White,
            texture: Some(texture),
        }
    }
}
