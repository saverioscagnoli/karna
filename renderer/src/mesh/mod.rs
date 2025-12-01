pub mod transform;
use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

use common::utils;
use math::{Vector2, Vector3, Vector4};
use wgpu::{RenderPass, util::DeviceExt};

use crate::{Color, Renderer, Transform2D};

pub trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    pub position: Vector3,
    pub color: Vector4,
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

#[derive(Debug, Clone, Copy)]
pub struct MeshInstance {
    pub position: Vector3,
    pub scale: Vector3,
    pub rotation: Vector3,
    pub color: Color,
}

impl MeshInstance {
    pub fn to_gpu(&self) -> MeshInstanceGPU {
        MeshInstanceGPU {
            position: self.position,
            scale: self.scale,
            rotation: self.rotation,
            color: self.color.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshInstanceGPU {
    pub position: Vector3,
    pub scale: Vector3,
    pub rotation: Vector3,
    pub color: Vector4,
}

impl Descriptor for MeshInstanceGPU {
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
                    offset: std::mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // rotation
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() * 2) as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() * 3) as wgpu::BufferAddress,
                    shader_location: 5,
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
    pub instances: Vec<MeshInstanceGPU>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshId(TypeId);

impl MeshId {
    pub fn of<M: Mesh + 'static>() -> Self {
        Self(TypeId::of::<M>())
    }
}

pub trait Mesh: Sized + 'static {
    const INITIAL_INSTANCE_CAPACITY: usize = 64;

    fn vertices(&self) -> &[Vertex];
    fn indices(&self) -> &[u32];
    fn instance(&self) -> &MeshInstance;

    fn render(&self, renderer: &mut Renderer) {
        renderer.draw_instance(self, self.instance());
    }
}

pub struct Rectangle {
    vertices: [Vertex; 4],
    indices: [u32; 6],
    pub instance: MeshInstance,
}

impl Rectangle {
    pub fn new(width: f32, height: f32, color: Color) -> Self {
        let vertices = [
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                color: color.into(),
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                color: color.into(),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                color: color.into(),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                color: color.into(),
            },
        ];

        let indices = [0, 1, 2, 0, 2, 3];

        let instance = MeshInstance {
            position: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(width, height, 1.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            color,
        };

        Self {
            vertices,
            indices,
            instance,
        }
    }
}

impl Mesh for Rectangle {
    fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    fn indices(&self) -> &[u32] {
        &self.indices
    }

    fn instance(&self) -> &MeshInstance {
        &self.instance
    }
}
