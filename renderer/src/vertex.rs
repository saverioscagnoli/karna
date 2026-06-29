use std::mem;

use math::Vector2;
use math::Vector3;
use math::Vector4;

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector4<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub fn new(p: Vector3<f32>, color: Vector4<f32>, uv: Vector2<f32>) -> Self {
        Self {
            position: p,
            color,
            uv,
        }
    }

    pub fn with_position(mut self, p: Vector3<f32>) -> Self {
        self.position = p;
        self
    }

    pub fn set_position(&mut self, p: Vector3<f32>) {
        self.position = p;
    }

    pub fn with_color(mut self, c: Vector4<f32>) -> Self {
        self.color = c;
        self
    }

    pub fn set_color(&mut self, c: Vector4<f32>) {
        self.color = c;
    }

    pub fn with_uv(mut self, uv: Vector2<f32>) -> Self {
        self.uv = uv;
        self
    }

    pub fn set_uv(&mut self, uv: Vector2<f32>) {
        self.uv = uv;
    }
}

impl Vertex {
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
                    offset: mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress
                        + mem::size_of::<Vector4<f32>>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
