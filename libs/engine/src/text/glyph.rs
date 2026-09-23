use core::ops::Range;

use sdl3::render::Color;

use crate::assets::Image;

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub image: Option<Image>,
    pub pos: math::Vector2<f32>,
    pub pen: math::Vector2<f32>,
    pub size: math::Size<u32>,
    pub range: Range<usize>,
    pub line: usize,
    pub color: Option<Color>,
    pub colored: bool,
    pub metadata: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct CachedGlyph {
    pub image: Image,
    pub placement: math::Vector2<f32>,
    pub size: math::Size<u32>,
    pub colored: bool,
}
