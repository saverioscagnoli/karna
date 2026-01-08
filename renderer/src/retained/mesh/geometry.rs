use crate::vertex::Vertex;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use math::{Size, Vector2, Vector3, Vector4};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::{Arc, RwLock},
};
use utils::FastHashMap;

static GEOMETRY_CACHE: RwLock<Option<FastHashMap<u64, Arc<GeometryBuffer>>>> = RwLock::new(None);

fn cache() -> &'static RwLock<Option<FastHashMap<u64, Arc<GeometryBuffer>>>> {
    {
        let read = GEOMETRY_CACHE.read().unwrap();
        if read.is_some() {
            drop(read);
            return &GEOMETRY_CACHE;
        }
    }

    let mut write = GEOMETRY_CACHE.write().unwrap();

    if write.is_none() {
        *write = Some(FastHashMap::default());
    }

    &GEOMETRY_CACHE
}

fn get_or_insert(id: u64, f: impl FnOnce() -> GeometryBuffer) -> Arc<GeometryBuffer> {
    {
        let read = cache().read().unwrap();
        if let Some(buffer) = read.as_ref().unwrap().get(&id) {
            return buffer.clone();
        }
    }

    let mut write = cache().write().unwrap();
    let map = write.as_mut().unwrap();

    if let Some(buffer) = map.get(&id) {
        return buffer.clone();
    }

    let buffer = Arc::new(f());

    map.insert(id, buffer.clone());

    buffer
}

#[derive(Debug)]
pub struct GeometryBuffer {
    pub vertex_buffer: GpuBuffer<Vertex>,
    pub index_buffer: GpuBuffer<u32>,
}

impl GeometryBuffer {
    pub fn new(vertices: &[Vertex], indices: &[u32]) -> Self {
        let vertex_buffer = GpuBufferBuilder::new()
            .label("Geometry Vertex Buffer")
            .vertex()
            .copy_dst()
            .data(vertices)
            .build();

        let index_buffer = GpuBufferBuilder::new()
            .label("Geometry Index Buffer")
            .index()
            .copy_dst()
            .data(indices)
            .build();

        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub id: u64,
    pub buffer: Arc<GeometryBuffer>,
}

impl Geometry {
    fn hash(vertices: &Vec<Vertex>, indices: &Vec<u32>) -> u64 {
        let mut hasher = DefaultHasher::new();

        vertices.hash(&mut hasher);
        indices.hash(&mut hasher);

        hasher.finish()
    }

    pub fn rect<S>(size: S) -> Self
    where
        S: Into<Size<f32>>,
    {
        let size: Size<f32> = size.into();
        let hw = size.width / 2.0;
        let hh = size.height / 2.0;

        let vertices = vec![
            Vertex::new(
                Vector3::new(-hw, -hh, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(hw, -hh, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(hw, hh, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(-hw, hh, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 1.0),
            ),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        let id = Self::hash(&vertices, &indices);
        let buffer = get_or_insert(id, || GeometryBuffer::new(&vertices, &indices));

        Self { id, buffer }
    }

    pub fn circle(radius: f32, segments: u32) -> Self {
        let mut vertices = Vec::with_capacity((segments + 1) as usize);
        let mut indices = Vec::with_capacity((segments * 3) as usize);

        vertices.push(Vertex::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector4::new(1.0, 1.0, 1.0, 1.0),
            Vector2::new(0.5, 0.5),
        ));

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * radius;
            let y = angle.sin() * radius;

            vertices.push(Vertex::new(
                Vector3::new(x, y, 0.0),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new((x / radius + 1.0) * 0.5, (y / radius + 1.0) * 0.5),
            ));
        }

        for i in 0..segments {
            indices.push(0);
            indices.push(i + 1);
            indices.push(if i + 2 > segments { 1 } else { i + 2 });
        }

        let id = Self::hash(&vertices, &indices);
        let buffer = get_or_insert(id, || GeometryBuffer::new(&vertices, &indices));
        Self { id, buffer }
    }

    pub fn cube(size: f32) -> Self {
        let s = size / 2.0;

        let vertices = vec![
            // Front face
            Vertex::new(
                Vector3::new(-s, -s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(s, -s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(s, s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(-s, s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 1.0),
            ),
            // Back face
            Vertex::new(
                Vector3::new(s, -s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(-s, -s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(-s, s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(s, s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 1.0),
            ),
            // Top face
            Vertex::new(
                Vector3::new(-s, s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(s, s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(s, s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(-s, s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 1.0),
            ),
            // Bottom face
            Vertex::new(
                Vector3::new(-s, -s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(s, -s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(s, -s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(-s, -s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 1.0),
            ),
            // Right face
            Vertex::new(
                Vector3::new(s, -s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(s, -s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(s, s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(s, s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 1.0),
            ),
            // Left face
            Vertex::new(
                Vector3::new(-s, -s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(-s, -s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(-s, s, s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(1.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(-s, s, -s),
                Vector4::new(1.0, 1.0, 1.0, 1.0),
                Vector2::new(0.0, 1.0),
            ),
        ];

        let indices = vec![
            0, 1, 2, 2, 3, 0, // Front
            4, 5, 6, 6, 7, 4, // Back
            8, 9, 10, 10, 11, 8, // Top
            12, 13, 14, 14, 15, 12, // Bottom
            16, 17, 18, 18, 19, 16, // Right
            20, 21, 22, 22, 23, 20, // Left
        ];

        let id = Self::hash(&vertices, &indices);
        let buffer = get_or_insert(id, || GeometryBuffer::new(&vertices, &indices));

        Self { id, buffer }
    }

    pub fn unit_rect() -> Self {
        Self::rect((1.0, 1.0))
    }

    pub fn new(vertices: &Vec<Vertex>, indices: &Vec<u32>) -> Self {
        let id = Self::hash(vertices, indices);
        let buffer = get_or_insert(id, || GeometryBuffer::new(vertices, indices));

        Self { id, buffer }
    }
}
