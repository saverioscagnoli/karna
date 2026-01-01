use crate::{
    Color,
    mesh::{GeometryBuffer, Vertex},
};
use globals::profiling;
use gpu::core::GpuBufferBuilder;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, LazyLock, RwLock},
};
use wgpu::naga::FastHashMap;

static GEOMETRY_CACHE: LazyLock<RwLock<FastHashMap<u32, Arc<GeometryBuffer>>>> =
    LazyLock::new(|| RwLock::new(FastHashMap::default()));

#[derive(Debug, Clone)]
pub struct Geometry {
    id: u32,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Geometry {
    fn compute_hash(vertices: &[Vertex], indices: &[u32]) -> u32 {
        let mut hasher = DefaultHasher::new();

        for vertex in vertices {
            vertex.position[0].to_bits().hash(&mut hasher);
            vertex.position[1].to_bits().hash(&mut hasher);
            vertex.position[2].to_bits().hash(&mut hasher);
        }

        indices.hash(&mut hasher);
        hasher.finish() as u32
    }

    #[inline]
    pub fn new(vertices: &[Vertex], indices: &[u32]) -> Self {
        let id = Self::compute_hash(vertices, indices);
        Self {
            id,
            vertices: vertices.to_vec(),
            indices: indices.to_vec(),
        }
    }

    /// Get or create the GPU buffer for this geometry
    #[inline]
    pub fn buffer(&self) -> Arc<GeometryBuffer> {
        {
            let cache = GEOMETRY_CACHE.read().unwrap();

            if let Some(buffer) = cache.get(&self.id) {
                return Arc::clone(buffer);
            }
        }

        // Not in cache, create new buffer
        let vertex_buffer = GpuBufferBuilder::new()
            .label("geometry vertex buffer")
            .vertex()
            .copy_dst()
            .data(self.vertices.to_vec())
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("geometry index buffer")
            .index()
            .copy_dst()
            .data(self.indices.to_vec())
            .build();

        let gpu_buffer = Arc::new(GeometryBuffer {
            vertex_buffer,
            vertex_count: self.vertices.len() as i32,
            index_buffer,
            index_count: self.indices.len() as i32,
            topology: wgpu::PrimitiveTopology::TriangleList,
        });

        // Cache it
        {
            let mut cache = GEOMETRY_CACHE.write().unwrap();
            profiling::record_geometry_buffer(1);
            profiling::record_geometry_buffers_size(
                (gpu_buffer.vertex_buffer.byte_size() + gpu_buffer.index_buffer.byte_size()) as u32,
            );
            cache.insert(self.id, Arc::clone(&gpu_buffer));
        }

        gpu_buffer
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn rect(width: f32, height: f32) -> Self {
        let color = Color::White.into();
        let vertices = &[
            Vertex {
                position: [0.0, 0.0, 0.0],
                color,
                uv_coords: [0.0, 0.0],
            },
            Vertex {
                position: [width, 0.0, 0.0],
                color,
                uv_coords: [1.0, 0.0],
            },
            Vertex {
                position: [width, height, 0.0],
                color,
                uv_coords: [1.0, 1.0],
            },
            Vertex {
                position: [0.0, height, 0.0],
                color,
                uv_coords: [0.0, 1.0],
            },
        ];

        let indices = &[0, 1, 2, 2, 3, 0];
        Self::new(vertices, indices)
    }

    #[inline]
    pub fn unit_rect() -> Self {
        Self::rect(1.0, 1.0)
    }

    #[inline]
    pub fn circle(r: f32, segments: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        vertices.push(Vertex {
            position: [0.0, 0.0, 0.0],
            color: Color::White.into(),
            uv_coords: [0.5, 0.5],
        });

        for i in 0..segments {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);

            let x = r * theta.cos();
            let y = r * theta.sin();

            vertices.push(Vertex {
                position: [x, y, 0.0],
                color: Color::White.into(),
                uv_coords: [0.5 + x / (2.0 * r), 0.5 + y / (2.0 * r)],
            });
        }

        for i in 1..segments {
            indices.push(0);
            indices.push(i);
            indices.push(i + 1);
        }

        indices.push(0);
        indices.push(segments);
        indices.push(1);

        Self::new(&vertices, &indices)
    }
}
