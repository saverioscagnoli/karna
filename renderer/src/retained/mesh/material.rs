use crate::color::Color;
use utils::Label;

#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    None,
    Full(Label),
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub color: Color,
    pub texture: TextureKind,
}
