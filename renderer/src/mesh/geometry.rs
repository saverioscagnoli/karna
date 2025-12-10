use crate::{Color, gpu, mesh::Vertex};
use math::{Vector2, Vector3, Vector4};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};
use traccia::debug;

#[derive(Debug, Clone)]
pub struct MeshGeometry {
    pub id: u32,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub topology: wgpu::PrimitiveTopology,
}

impl MeshGeometry {
    pub fn new(
        vertices: &[Vertex],
        indices: &[u32],
        topology: wgpu::PrimitiveTopology,
    ) -> Arc<Self> {
        let id = Self::compute_hash(vertices, indices, &topology);
        let gpu = gpu();

        // Try to get from cache first
        {
            let cache_guard = gpu.geometry_cache.load();

            if let Some(g) = cache_guard.get(&id) {
                return Arc::clone(g);
            }
        }

        // Update cache using RCU (Read-Copy-Update)
        let geometry = Arc::new(Self {
            id,
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
            topology,
        });

        gpu.geometry_cache.rcu(|cache| {
            let mut new_cache = (**cache).clone();
            new_cache.insert(id, Arc::clone(&geometry));
            new_cache
        });

        debug!(
            "Creating geometry with id '{}', n. vertices: {}, n. indices:  {}",
            id,
            vertices.len(),
            indices.len()
        );

        geometry
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
        let color: Vector4 = Color::White.into();

        let vertices = &[
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                color,
                uv: Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 0.0, 0.0),
                color,
                uv: Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(1.0, 1.0, 0.0),
                color,
                uv: Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, 1.0, 0.0),
                color,
                uv: Vector2::new(0.0, 1.0),
            },
        ];

        let indices = &[0, 1, 2, 2, 3, 0];
        Self::new(vertices, indices, wgpu::PrimitiveTopology::TriangleList)
    }

    pub fn pixel() -> Arc<Self> {
        let vertices = &[Vertex {
            position: Vector3::zeros(),
            color: Color::White.into(),
            uv: Vector2::zeros(),
        }];

        let indices = &[0];

        Self::new(vertices, indices, wgpu::PrimitiveTopology::PointList)
    }

    pub fn circle(radius: f32, segments: u32) -> Arc<Self> {
        let segments = segments.max(3);
        let num_vertices = 1 + segments as usize;
        let color: Vector4 = Color::White.into();

        let mut vertices = Vec::with_capacity(num_vertices);
        let mut indices = Vec::with_capacity(segments as usize * 3); // 3 indices per segment

        vertices.push(Vertex {
            position: Vector3::zeros(),
            color,
            uv: Vector2::new(0.5, 0.5),
        });

        let center_index: u32 = 0;
        let angle_step = std::f32::consts::TAU / segments as f32; // TAU = 2 * PI

        for i in 0..segments {
            let angle = i as f32 * angle_step;

            let x = radius * angle.cos();
            let y = radius * angle.sin();

            vertices.push(Vertex {
                position: Vector3::new(x, y, 0.0),
                color,
                uv: Vector2::new((x / radius + 1.0) * 0.5, (y / radius + 1.0) * 0.5),
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
