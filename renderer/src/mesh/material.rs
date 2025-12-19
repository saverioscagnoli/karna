use crate::Color;
use utils::map::Label;

#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    Full(Label),
    Partial(Label, f32, f32, f32, f32),
}

#[derive(Debug, Clone)]
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
    #[inline]
    pub fn new(color: Color, texture: TextureKind) -> Self {
        Self {
            color,
            texture: Some(texture),
        }
    }

    #[inline]
    pub fn new_texture(texture: TextureKind) -> Self {
        Self {
            color: Color::White,
            texture: Some(texture),
        }
    }

    #[inline]
    pub fn new_color(color: Color) -> Self {
        Self {
            color,
            texture: None,
        }
    }

    #[inline]
    pub fn with_texture(mut self, texture: TextureKind) -> Self {
        self.texture = Some(texture);
        self
    }
}
