use crate::Color;
use utils::map::Label;

#[derive(Debug, Clone)]
pub struct Material {
    pub color: Color,
    pub texture: Option<Label>,
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
    pub fn new(color: Color, texture: Option<Label>) -> Self {
        Self { color, texture }
    }

    #[inline]
    pub fn new_texture(label: Label) -> Self {
        Self {
            color: Color::White,
            texture: Some(label),
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
    pub fn with_texture(mut self, label: Label) -> Self {
        self.texture = Some(label);
        self
    }
}
