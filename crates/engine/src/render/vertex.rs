use sdl3::SDL_GPUVertexElementFormat;

use crate::gpu::LayoutDescriptor;
use crate::gpu::VertexAttribute;

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: math::Vector3<f32>,
    pub color: math::Vector4<f32>,
    pub uv: math::Vector2<f32>,
}

impl LayoutDescriptor for Vertex {
    const ATTRIBUTES: &'static [VertexAttribute] = &[
        VertexAttribute {
            location: 0,
            format: SDL_GPUVertexElementFormat::SDL_GPU_VERTEXELEMENTFORMAT_FLOAT3,
            offset: 0,
        },
        VertexAttribute {
            location: 1,
            format: SDL_GPUVertexElementFormat::SDL_GPU_VERTEXELEMENTFORMAT_FLOAT4,
            offset: 12,
        },
        VertexAttribute {
            location: 2,
            format: SDL_GPUVertexElementFormat::SDL_GPU_VERTEXELEMENTFORMAT_FLOAT2,
            offset: 28,
        },
    ];
}
