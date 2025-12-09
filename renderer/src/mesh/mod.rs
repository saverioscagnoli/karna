pub mod geometry;
pub mod material;
pub mod transform;

use crate::{
    Color, Renderer,
    mesh::{geometry::MeshGeometry, material::Material, transform::Transform},
};
use math::{Vector2, Vector3, Vector4};
use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

pub trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    position: Vector3,
    color: Vector4,
    uv: Vector2,
}

impl Descriptor for Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() + std::mem::size_of::<Vector4>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshBuffer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub instance_buffer: wgpu::Buffer,
    pub textured_instances: Vec<RawMesh>,
    pub untextured_instances: Vec<RawMesh>,
    pub topology: wgpu::PrimitiveTopology,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub geometry: Arc<MeshGeometry>,
    pub material: Material,
    pub transform: Transform,
}

impl Deref for Mesh {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl DerefMut for Mesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}

impl Mesh {
    pub(crate) const INITIAL_INSTANCE_CAPACITY: usize = 64;

    pub(crate) fn to_raw(&self) -> RawMesh {
        RawMesh {
            position: self.position.extend(0.0).into(),
            scale: self.scale.extend(1.0).into(),
            rotation: [0.0, 0.0, self.rotation],
            color: self.material.color.unwrap_or(Color::White).into(),
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.draw_mesh(self);
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Represents the higher-level mesh instance for the gpu.
pub struct RawMesh {
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub rotation: [f32; 3],
    pub color: [f32; 4],
    pub uv_offset: [f32; 2],
    pub uv_scale: [f32; 2],
}

impl Descriptor for RawMesh {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // scale
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // rotation
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 3) as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // uv_offset
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 3 + std::mem::size_of::<[f32; 4]>())
                        as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // uv_scale
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 3
                        + std::mem::size_of::<[f32; 4]>()
                        + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
