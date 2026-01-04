use crate::{retained::RetainedRenderer, vertex::Vertex};
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use logging::info;
use math::{Vector2, Vector3, Vector4};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

#[derive(Debug)]
pub struct GeometryBuffer {
    pub vertex_buffer: GpuBuffer<Vertex>,
    pub index_buffer: GpuBuffer<u32>,
}

impl GeometryBuffer {
    pub fn new(id: u64, vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Self {
        let vertex_buffer = GpuBufferBuilder::new()
            .label(&format!("Geometry Vertex Buffer {}", id))
            .vertex()
            .copy_dst()
            .data(vertices)
            .build();
        let index_buffer = GpuBufferBuilder::new()
            .label(&format!("Geometry Index Buffer {}", id))
            .index()
            .copy_dst()
            .data(indices)
            .build();

        GeometryBuffer {
            vertex_buffer,
            index_buffer,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub buffer: Arc<GeometryBuffer>,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Geometry {
    fn compute_hash(vertices: &Vec<Vertex>, indices: &Vec<u32>) -> u64 {
        let mut hasher = DefaultHasher::new();

        vertices.hash(&mut hasher);
        indices.hash(&mut hasher);

        hasher.finish()
    }

    fn new<'a>(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        renderer: &'a mut RetainedRenderer,
    ) -> Self {
        let id = Self::compute_hash(&vertices, &indices);
        let buffer = match renderer.geometry_cache.get(&id) {
            Some(b) => b.clone(),
            None => {
                info!(
                    "Creating new geometry with id: {}, n.vertices: {}, n.indices: {}",
                    id,
                    vertices.len(),
                    indices.len()
                );

                Arc::new(GeometryBuffer::new(id, &vertices, &indices))
            }
        };

        Geometry {
            buffer,
            vertices,
            indices,
        }
    }
}

pub struct GeometryBuilder<'a> {
    renderer: &'a mut RetainedRenderer,
}

impl<'a> GeometryBuilder<'a> {
    pub fn new(renderer: &'a mut RetainedRenderer) -> Self {
        GeometryBuilder { renderer }
    }

    pub fn rect(self) -> Geometry {
        let vertices = vec![
            Vertex::new(
                Vector3::new(0.0, 0.0, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(1.0, 0.0, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(1.0, 1.0, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(0.0, 1.0, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 1.0),
            ),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        Geometry::new(vertices, indices, self.renderer)
    }
}
