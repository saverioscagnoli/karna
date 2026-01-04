use crate::color::Color;
use assets::Image;
use utils::{Handle, Label};

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
