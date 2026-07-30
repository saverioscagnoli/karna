use std::mem;

pub use sdl3::gpu::VertexElementFormat;

/// A single vertex attribute: shader location, element format and byte offset
/// within the vertex struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexAttribute {
    pub location: u32,
    pub format: VertexElementFormat,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VertexLayout {
    pub pitch: u32,
    pub attributes: &'static [VertexAttribute],
}

/// Describes how a vertex struct maps to shader inputs.
pub trait LayoutDesc: Sized {
    const ATTRIBUTES: &'static [VertexAttribute];

    fn desc() -> VertexLayout {
        VertexLayout {
            pitch: mem::size_of::<Self>() as u32,
            attributes: Self::ATTRIBUTES,
        }
    }
}
