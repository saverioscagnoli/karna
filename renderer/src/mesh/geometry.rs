use crate::mesh::{GeometryBuffer, Vertex};
use gpu::core::GpuBufferBuilder;
use math::{Size, Vector3, Vector4};
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
            vertex.position.x.to_bits().hash(&mut hasher);
            vertex.position.y.to_bits().hash(&mut hasher);
            vertex.position.z.to_bits().hash(&mut hasher);
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
            cache.insert(self.id, Arc::clone(&gpu_buffer));
        }

        gpu_buffer
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn rect<S: Into<Size<f32>>>(size: S) -> Self {
        let size = size.into();
        let color = Vector4::ones();
        let vertices = &[
            Vertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                color,
                uv_coords: math::Vector2::new(0.0, 0.0),
            },
            Vertex {
                position: Vector3::new(size.width, 0.0, 0.0),
                color,
                uv_coords: math::Vector2::new(1.0, 0.0),
            },
            Vertex {
                position: Vector3::new(size.width, size.height, 0.0),
                color,
                uv_coords: math::Vector2::new(1.0, 1.0),
            },
            Vertex {
                position: Vector3::new(0.0, size.height, 0.0),
                color,
                uv_coords: math::Vector2::new(0.0, 1.0),
            },
        ];

        let indices = &[0, 1, 2, 2, 3, 0];
        Self::new(vertices, indices)
    }

    #[inline]
    pub fn unit_rect() -> Self {
        Self::rect(Size::new(1.0, 1.0))
    }
}
