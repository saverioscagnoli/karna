use std::mem;

use sdl3::SDL_GPUVertexElementFormat;

pub struct VertexAttribute {
    pub location: u32,
    pub format: SDL_GPUVertexElementFormat,
    pub offset: u32,
}

pub struct VertexLayout {
    pub pitch: u32,
    pub attributes: &'static [VertexAttribute],
}

pub trait LayoutDescriptor: Sized {
    const ATTRIBUTES: &'static [VertexAttribute];

    fn desc() -> VertexLayout {
        VertexLayout {
            pitch: mem::size_of::<Self>() as u32,
            attributes: Self::ATTRIBUTES,
        }
    }
}
