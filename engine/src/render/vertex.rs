use std::mem;

/// Describes how a vertex struct maps to shader inputs.
pub trait LayoutDesc: Sized {
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
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    pub position: math::Vector3<f32>,
    pub color: math::Vector4<f32>,
    pub uv: math::Vector2<f32>,
}

impl LayoutDesc for Vertex {
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

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshVertex {
    pub position: math::Vector3<f32>, // Offset 0
    pub normal: math::Vector3<f32>,   // Offset 12
    pub uv: math::Vector2<f32>,       // Offset 24
}

impl LayoutDesc for MeshVertex {
    const ATTRIBUTES: &'static [gpu::VertexAttribute] = &[
        gpu::VertexAttribute {
            location: 0,
            format: gpu::VertexElementFormat::Float3,
            offset: 0,
        },
        gpu::VertexAttribute {
            location: 1,
            format: gpu::VertexElementFormat::Float3,
            offset: 12,
        },
        gpu::VertexAttribute {
            location: 2,
            format: gpu::VertexElementFormat::Float2,
            offset: 24,
        },
    ];
}
