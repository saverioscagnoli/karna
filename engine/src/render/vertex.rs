use std::mem;

pub trait VertexLayoutDescriptor: Sized {
    const ATTRIBUTES: &'static [gpu::VertexAttribute];

    fn desc() -> gpu::VertexLayout {
        gpu::VertexLayout {
            pitch: mem::size_of::<Self>() as u32,
            attributes: Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: math::Vector3<f32>,
    pub color: math::Vector4<f32>,
    pub uv: math::Vector2<f32>,
}

impl VertexLayoutDescriptor for Vertex {
    const ATTRIBUTES: &'static [gpu::VertexAttribute] = &[
        gpu::VertexAttribute {
            location: 0,
            format: gpu::VertexElementFormat::Float3,
            offset: 0,
        },
        gpu::VertexAttribute {
            location: 1,
            format: gpu::VertexElementFormat::Float4,
            offset: 12,
        },
        gpu::VertexAttribute {
            location: 2,
            format: gpu::VertexElementFormat::Float2,
            offset: 28,
        },
    ];
}
