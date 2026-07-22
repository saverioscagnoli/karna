use std::ops::Range;

/// What a batch of immediate-mode geometry samples from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchTexture {
    /// A page of the shared atlas. Page 0 always exists and holds the white
    /// pixel, so untextured geometry uses it with the white pixel's uvs.
    Page(usize),
    /// An offscreen canvas' render target, by canvas id.
    Canvas(u64),
}

impl BatchTexture {
    /// The page holding the white pixel, used by untextured geometry.
    pub const WHITE: Self = Self::Page(0);
}

impl Default for BatchTexture {
    fn default() -> Self {
        Self::WHITE
    }
}

#[derive(Debug, Clone)]
pub struct Batch {
    pub indices: Range<u32>,
    pub texture: BatchTexture,
}
