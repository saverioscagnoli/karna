pub mod geometry;
pub mod transform;

use crate::{
    Color, Renderer,
    mesh::{geometry::MeshGeometry, transform::Transform},
};
use math::{Vector3, Vector4};
use std::{
    hash::{Hash, Hasher},
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
    pub instances: Vec<RawMesh>,
    pub topology: wgpu::PrimitiveTopology,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub geometry: Arc<MeshGeometry>,
    pub color: Color,
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
            color: self.color.into(),
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
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // scale
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // rotation
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() * 3) as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
