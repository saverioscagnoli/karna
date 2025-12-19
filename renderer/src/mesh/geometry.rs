use crate::{Color, Vertex};
use macros::Get;
use math::{Vector3, Vector4};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, LazyLock, RwLock},
};
use traccia::info;
use wgpu::naga::FastHashMap;

static GEOMETRY_CACHE: LazyLock<RwLock<FastHashMap<u32, Arc<Geometry>>>> =
    LazyLock::new(|| RwLock::new(FastHashMap::default()));

#[derive(Debug)]
#[derive(Get)]
pub struct Geometry {
    #[get(copied)]
    id: u32,

    #[get]
    vertices: Vec<Vertex>,

    #[get]
    indices: Vec<u32>,

    #[get(copied)]
    topology: wgpu::PrimitiveTopology,
}

impl Geometry {
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

    #[inline]
    pub fn new(
        vertices: &[Vertex],
        indices: &[u32],
        topology: wgpu::PrimitiveTopology,
    ) -> Arc<Self> {
        let id = Self::compute_hash(vertices, indices, &topology);

        {
            let lock = GEOMETRY_CACHE
                .read()
                .expect("Geometry cache lock is poisoned");

            if let Some(geometry) = lock.get(&id) {
                return Arc::clone(geometry);
            }
        }

        let mut lock = GEOMETRY_CACHE
            .write()
            .expect("Geometry cache lock is poisoned");

        let geometry = Arc::new(Self {
            id,
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            topology,
        });

        info!(
            "Created new geometry with {} vertices and {} indices with id '{}'",
            vertices.len(),
            indices.len(),
            id
        );

        lock.insert(id, Arc::clone(&geometry));

        geometry
    }

    #[inline]
    pub fn unit_rect() -> Arc<Self> {
        Self::rect(1.0, 1.0)
    }

    #[inline]
    pub fn rect(w: f32, h: f32) -> Arc<Self> {
        let color: Vector4 = Color::White.into();

        let vertices = &[
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                color,
                uv_coords: math::Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(w, 0.0, 0.0),
                color,
                uv_coords: math::Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(w, h, 0.0),
                color,
                uv_coords: math::Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, h, 0.0),
                color,
                uv_coords: math::Vector2::new(0.0, 1.0),
            },
        ];

        let indices = &[0, 1, 2, 2, 3, 0];
        Self::new(vertices, indices, wgpu::PrimitiveTopology::TriangleList)
    }

    #[inline]
    pub fn cube() -> Arc<Self> {
        let color: Vector4 = Color::White.into();

        let vertices = &[
            // Front face (z = 1.0)
            Vertex {
                position: Vector3::new(0.0, 0.0, 1.0),
                color,
                uv_coords: math::Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 1.0),
                color,
                uv_coords: math::Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 1.0),
                color,
                uv_coords: math::Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 1.0),
                color,
                uv_coords: math::Vector2::new(0.0, 1.0),
            },
            // Back face (z = 0.0)
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                color,
                uv_coords: math::Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                color,
                uv_coords: math::Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                color,
                uv_coords: math::Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                color,
                uv_coords: math::Vector2::new(0.0, 1.0),
            },
        ];

        let indices = &[
            // Front face
            0, 1, 2, 2, 3, 0, // Back face
            5, 4, 7, 7, 6, 5, // Left face
            4, 0, 3, 3, 7, 4, // Right face
            1, 5, 6, 6, 2, 1, // Top face
            3, 2, 6, 6, 7, 3, // Bottom face
            4, 5, 1, 1, 0, 4,
        ];

        Self::new(vertices, indices, wgpu::PrimitiveTopology::TriangleList)
    }

    #[inline]
    pub fn pixel() -> Arc<Self> {
        let vertices = &[Vertex {
            position: Vector3::zeros(),
            color: Color::White.into(),
            uv_coords: math::Vector2::new(0.0, 0.0),
        }];

        let indices = &[0];

        Self::new(vertices, indices, wgpu::PrimitiveTopology::PointList)
    }
}
