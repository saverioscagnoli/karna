use crate::{Renderer, Vertex};
use macros::impl_mesh_deref;
use nalgebra::{Quaternion, UnitQuaternion, Vector3, Vector4};
use std::{
    any::TypeId,
    ops::{Deref, DerefMut},
};

pub struct MeshData {
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,
    pub(crate) index_count: u32,
}

pub trait Mesh: DerefMut<Target = InstanceData> + Sized + 'static {
    fn vertices() -> Vec<Vertex>;
    fn indices() -> Vec<u32>;

    fn new() -> Self
    where
        Self: Default,
    {
        Self::default()
    }

    fn position(&self) -> &Vector3<f32> {
        &self.position
    }

    fn with_position(mut self, position: Vector3<f32>) -> Self {
        self.position = position;
        self
    }

    fn with_position_x(mut self, x: f32) -> Self {
        self.position.x = x;
        self
    }

    fn with_position_y(mut self, y: f32) -> Self {
        self.position.y = y;
        self
    }

    fn with_position_z(mut self, z: f32) -> Self {
        self.position.z = z;
        self
    }

    fn scale(&self) -> &Vector3<f32> {
        &self.scale
    }

    fn with_scale(mut self, scale: Vector3<f32>) -> Self {
        self.scale = scale;
        self
    }

    fn with_scale_x(mut self, x: f32) -> Self {
        self.scale.x = x;
        self
    }

    fn with_scale_y(mut self, y: f32) -> Self {
        self.scale.y = y;
        self
    }

    fn with_scale_z(mut self, z: f32) -> Self {
        self.scale.z = z;
        self
    }

    fn color(&self) -> &Vector4<f32> {
        &self.color
    }

    fn with_color(mut self, color: Vector4<f32>) -> Self {
        self.color = color;
        self
    }

    fn rotation(&self) -> &Vector3<f32> {
        &self.rotation
    }

    fn with_rotation(mut self, rotation: Vector3<f32>) -> Self {
        self.rotation = rotation;
        self
    }

    fn with_rotation_x(mut self, x: f32) -> Self {
        self.rotation.x = x;
        self
    }

    fn with_rotation_y(mut self, y: f32) -> Self {
        self.rotation.y = y;
        self
    }

    fn with_rotation_z(mut self, z: f32) -> Self {
        self.rotation.z = z;
        self
    }

    fn render(&self, renderer: &mut Renderer) {
        renderer.draw_mesh::<Self>(&self);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshId(TypeId);

impl MeshId {
    pub fn of<M: Mesh + 'static>() -> Self {
        MeshId(TypeId::of::<M>())
    }
}

/// Data for each instance of a mesh.
///
/// This is the struct that will stay on the cpu side,
/// different from `InstanceDataGpu` because I want to expose
/// a Vector4 instead of a Quaternion for rotations,
/// so it can be easily modified with `mesh.rotation.x|y|z += 1.0`
#[derive(Debug, Clone, Copy)]
pub struct InstanceData {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub color: Vector4<f32>,
}

/// This is the actual struct that will be sent
/// to the shader
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InstanceDataGpu {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
    pub color: Vector4<f32>,
}

impl Default for InstanceData {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            rotation: Vector3::zeros(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            color: Vector4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl InstanceData {
    pub fn to_gpu(&self) -> InstanceDataGpu {
        let rotation_quat =
            UnitQuaternion::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z)
                .into_inner();

        InstanceDataGpu {
            position: self.position,
            rotation: rotation_quat,
            scale: self.scale,
            color: self.color,
        }
    }
}

impl InstanceDataGpu {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceDataGpu>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Rotation (quaternion)
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // Scale
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3<f32>>()
                        + std::mem::size_of::<Quaternion<f32>>())
                        as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Color
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3<f32>>() * 2
                        + std::mem::size_of::<Quaternion<f32>>())
                        as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Debug, Default)]
pub struct Rectangle {
    pub instance_data: InstanceData,
}

impl_mesh_deref!(Rectangle);

impl Mesh for Rectangle {
    fn vertices() -> Vec<Vertex> {
        vec![
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
        ]
    }

    fn indices() -> Vec<u32> {
        vec![0, 1, 2, 1, 3, 2]
    }
}

#[derive(Debug, Default)]
pub struct Cube {
    pub instance_data: InstanceData,
}

impl_mesh_deref!(Cube);

impl Mesh for Cube {
    fn vertices() -> Vec<Vertex> {
        vec![
            // Front face (z = 0.5)
            Vertex {
                position: Vector3::new(-0.5, -0.5, 0.5),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.5, -0.5, 0.5),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.5, 0.5, 0.5),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(-0.5, 0.5, 0.5),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            // Back face (z = -0.5)
            Vertex {
                position: Vector3::new(-0.5, -0.5, -0.5),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.5, -0.5, -0.5),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.5, 0.5, -0.5),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(-0.5, 0.5, -0.5),
                color: Vector4::new(1.0, 1.0, 1.0, 1.0),
            },
        ]
    }

    fn indices() -> Vec<u32> {
        vec![
            // Front face
            0, 1, 2, 2, 3, 0, // Right face
            1, 5, 6, 6, 2, 1, // Back face
            5, 4, 7, 7, 6, 5, // Left face
            4, 0, 3, 3, 7, 4, // Top face
            3, 2, 6, 6, 7, 3, // Bottom face
            4, 5, 1, 1, 0, 4,
        ]
    }
}
