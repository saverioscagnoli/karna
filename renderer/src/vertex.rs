use crate::traits::LayoutDescriptor;
use math::{Vector2, Vector3, Vector4};
use std::{
    hash::{Hash, Hasher},
    mem,
};

/// General purpose vertex, used in Mesh rendering
/// and in immediate mode, but not in text / circles
/// or shapes that require special attention
#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    pub position: Vector3,
    pub color: Vector4,
    pub uv: Vector2,
}

impl Vertex {
    #[inline]
    pub fn new(position: Vector3, color: Vector4, uv: Vector2) -> Self {
        Vertex {
            position,
            color,
            uv,
        }
    }
}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position.x.to_bits().hash(state);
        self.position.y.to_bits().hash(state);
        self.position.z.to_bits().hash(state);

        self.color.x.to_bits().hash(state);
        self.color.y.to_bits().hash(state);
        self.color.z.to_bits().hash(state);
        self.color.w.to_bits().hash(state);

        self.uv.x.to_bits().hash(state);
        self.uv.y.to_bits().hash(state);
    }
}

impl LayoutDescriptor for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vector3>() as wgpu::BufferAddress
                        + mem::size_of::<Vector4>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Vertex type Specifically used for rendering
/// circles in immediate mode via `draw.cirlce()`
///
/// Uses a shader for cutting out pixels and make it
/// into a circle.
#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CircleVertex {
    pub position: Vector3, // 12 bytes
    pub color: Vector4,    // 16 bytes
    pub center: Vector2,   // 8 bytes
    pub radius: f32,       // 4 bytes
}

impl CircleVertex {
    #[inline]
    pub fn new(position: Vector3, color: Vector4, center: Vector2, radius: f32) -> Self {
        CircleVertex {
            position,
            color,
            center,
            radius,
        }
    }
}

impl LayoutDescriptor for CircleVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position: vec3<f32>
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color: vec4<f32>
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // center: vec2<f32>
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vector3>() + mem::size_of::<Vector4>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // radius: f32
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vector3>()
                        + mem::size_of::<Vector4>()
                        + mem::size_of::<Vector2>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
