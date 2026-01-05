use fontdue::Font;
use math::{Vector2, Vector3, Vector4};
use utils::Handle;

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    character: char,
    /// Position relative to the origin of the text
    position: Vector2,
    uv_offset: Vector2,
    uv_scale: Vector2,
    size: Vector2,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlyphGpu {
    position: Vector2,
    size: Vector2,
    uv_offset: Vector2,
    uv_scale: Vector2,
    color: Vector4,
}

pub struct Text {
    content: String,
    font: Handle<Font>,
}
