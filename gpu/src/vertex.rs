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
