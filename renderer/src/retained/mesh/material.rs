use crate::color::Color;
use assets::Image;
use utils::Handle;

#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    None,
    Full(Handle<Image>),
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub texture: TextureKind,
}

impl Material {
    #[inline]
    pub fn new(color: Color, texture: TextureKind) -> Self {
        Self { color, texture }
    }

    #[inline]
    pub fn new_color(color: Color) -> Self {
        Self {
            color,
            texture: TextureKind::None,
        }
    }

    #[inline]
    pub fn new_texture(texture: TextureKind) -> Self {
        Self {
            color: Color::White,
            texture,
        }
    }
}
