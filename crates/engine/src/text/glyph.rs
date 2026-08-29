use std::ops::Range;

use math as m;

use crate::Color;
use crate::Image;

#[derive(Debug, Clone)]
pub struct PositionedGlyph {
    pub image: Option<Image>,
    pub pos: m::Vector2<f32>,
    pub pen: m::Vector2<f32>,
    pub size: m::Size<u32>,
    pub range: Range<usize>,
    pub line: usize,
    pub color: Option<Color>,
    pub colored: bool,
    pub metadata: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct CachedGlyph {
    pub image: Image,
    pub placement: m::Vector2<f32>,
    pub size: m::Size<u32>,
    pub colored: bool,
}
