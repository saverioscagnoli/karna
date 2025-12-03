pub mod dirty;
pub mod material;
pub mod transform;

use crate::{Color, Renderer, Transform2D, material::Material, mesh::dirty::DirtyTracked};
use math::{Vector2, Vector3, Vector4};
use mini_moka::sync::Cache;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::{Deref, DerefMut},
    sync::{Arc, LazyLock},
};
use traccia::debug;

pub trait Descriptor {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    pub position: Vector3,
    pub uv: Vector2,
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
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeshInstance {
    pub transform: Transform2D,
    pub color: Color,
    pub uv_offset: Vector2,
    pub uv_scale: Vector2,
}

impl Default for MeshInstance {
    fn default() -> Self {
        Self {
            transform: Transform2D::default(),
            color: Color::White,
            uv_offset: Vector2::zeros(),
            uv_scale: Vector2::new(1.0, 1.0),
        }
    }
}

impl Deref for MeshInstance {
    type Target = Transform2D;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl DerefMut for MeshInstance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}

impl MeshInstance {
    pub fn to_gpu(&self) -> MeshInstanceGPU {
        MeshInstanceGPU {
            position: self.transform.position.extend(0.0),
            scale: self.transform.scale.extend(1.0),
            rotation: Vector3::new(0.0, 0.0, self.transform.rotation),
            color: self.color.into(),
            uv_offset: self.uv_offset,
            uv_scale: self.uv_scale,
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
    pub uv_offset: Vector2,
    pub uv_scale: Vector2,
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
                // uv_offset
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() * 3 + std::mem::size_of::<Vector4>())
                        as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // uv_scale
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<Vector3>() * 3
                        + std::mem::size_of::<Vector4>()
                        + std::mem::size_of::<Vector2>())
                        as wgpu::BufferAddress,
                    shader_location: 7,
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
    pub instances: Vec<MeshInstanceGPU>,
    pub topology: wgpu::PrimitiveTopology,
    pub material: Material,
}

#[derive(Debug, Clone)]
pub struct MeshGeometry {
    pub id: u32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub topology: wgpu::PrimitiveTopology,
}

static GEOMETRY_CACHE: LazyLock<Cache<u32, Arc<MeshGeometry>>> =
    LazyLock::new(|| Cache::new(Mesh::INITIAL_INSTANCE_CAPACITY as u64));

impl MeshGeometry {
    pub fn new(
        vertices: &[Vertex],
        indices: &[u32],
        topology: wgpu::PrimitiveTopology,
    ) -> Arc<Self> {
        let id = Self::compute_hash(vertices, indices, &topology);

        match GEOMETRY_CACHE.get(&id) {
            Some(g) => g,
            None => {
                let geometry = Arc::new(Self {
                    id,
                    vertices: vertices.to_vec(),
                    indices: indices.to_vec(),
                    topology,
                });

                GEOMETRY_CACHE.insert(id, geometry.clone());

                debug!(
                    "Creating geometry with id '{}', n. vertices: {}, n. indices:  {}",
                    id,
                    vertices.len(),
                    indices.len()
                );

                geometry
            }
        }
    }

    fn compute_hash(
        vertices: &[Vertex],
        indices: &[u32],
        topology: &wgpu::PrimitiveTopology,
    ) -> u32 {
        let mut hasher = DefaultHasher::new();

        for vertex in vertices {
            vertex.position.x.to_bits().hash(&mut hasher);
            vertex.position.y.to_bits().hash(&mut hasher);
            vertex.position.z.to_bits().hash(&mut hasher);
        }

        indices.hash(&mut hasher);
        std::mem::discriminant(topology).hash(&mut hasher);

        hasher.finish() as u32
    }

    pub fn rect() -> Arc<Self> {
        let vertices = &[
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                uv: Vector2::new(0.0, 1.0),
            },
        ];

        let indices = &[0, 1, 2, 2, 3, 0];
        Self::new(vertices, indices, wgpu::PrimitiveTopology::TriangleList)
    }

    pub fn pixel() -> Arc<Self> {
        let vertices = &[Vertex {
            position: Vector3::zeros(),
            uv: Vector2::zeros(),
        }];

        let indices = &[0];

        Self::new(vertices, indices, wgpu::PrimitiveTopology::PointList)
    }

    pub fn circle(radius: f32, segments: u32) -> Arc<Self> {
        let segments = segments.max(3);
        let num_vertices = 1 + segments as usize;

        let mut vertices = Vec::with_capacity(num_vertices);
        let mut indices = Vec::with_capacity(segments as usize * 3); // 3 indices per segment

        vertices.push(Vertex {
            position: Vector3::zeros(),
            uv: Vector2::new(0.5, 0.5),
        });

        let center_index: u32 = 0;
        let angle_step = std::f32::consts::TAU / segments as f32; // TAU = 2 * PI

        for i in 0..segments {
            let angle = i as f32 * angle_step;

            let x = radius * angle.cos();
            let y = radius * angle.sin();

            let uv_x = (x / (2.0 * radius)) + 0.5;
            let uv_y = (y / (2.0 * radius)) + 0.5;

            vertices.push(Vertex {
                position: Vector3::new(x, y, 0.0),
                uv: Vector2::new(uv_x, uv_y),
            });

            let current_vertex_index = i + 1;
            let next_vertex_index = if i == segments - 1 { 1 } else { i + 2 };

            indices.push(center_index);
            indices.push(current_vertex_index);
            indices.push(next_vertex_index);
        }

        Self::new(&vertices, &indices, wgpu::PrimitiveTopology::TriangleList)
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub(crate) geometry: Arc<MeshGeometry>,
    pub(crate) material: Material,
    pub instance: DirtyTracked<MeshInstance>,
}

impl Deref for Mesh {
    type Target = MeshInstance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl DerefMut for Mesh {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.instance
    }
}

impl Mesh {
    pub const INITIAL_INSTANCE_CAPACITY: usize = 64;

    pub fn new(geometry: Arc<MeshGeometry>, material: Material, transform: Transform2D) -> Self {
        Self {
            geometry,
            instance: MeshInstance {
                transform,
                color: material.tint,
                uv_offset: Vector2::zeros(),
                uv_scale: Vector2::new(1.0, 1.0),
            }
            .into(),
            material,
        }
    }

    /// Create a mesh with a specific atlas region for UV mapping
    pub fn with_atlas_region(
        geometry: Arc<MeshGeometry>,
        material: Material,
        transform: Transform2D,
        uv_offset: Vector2,
        uv_scale: Vector2,
    ) -> Self {
        Self {
            geometry,
            instance: MeshInstance {
                transform,
                color: material.tint,
                uv_offset,
                uv_scale,
            }
            .into(),
            material,
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer) {
        renderer.draw_instance(self);

        if self.instance.dirty {
            self.instance.dirty = false;
        }
    }
}
