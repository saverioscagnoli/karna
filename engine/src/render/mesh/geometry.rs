use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crate::render::vertex::MeshVertex;

static NEXT_GEOMETRY_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct Geometry {
    id: u64,
    version: u64,
    vertices: Arc<Vec<MeshVertex>>,
    indices: Arc<Vec<u32>>,
}

impl Geometry {
    pub fn new(vertices: Vec<MeshVertex>, indices: Vec<u32>) -> Self {
        Self {
            id: NEXT_GEOMETRY_ID.fetch_add(1, Ordering::Relaxed),
            version: 0,
            vertices: Arc::new(vertices),
            indices: Arc::new(indices),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn vertices(&self) -> &[MeshVertex] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn vertices_mut(&mut self) -> &mut Vec<MeshVertex> {
        self.version += 1;
        Arc::make_mut(&mut self.vertices)
    }

    pub fn indices_mut(&mut self) -> &mut Vec<u32> {
        self.version += 1;
        Arc::make_mut(&mut self.indices)
    }

    pub fn cube(size: f32) -> Self {
        let h = size * 0.5;

        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            ([0.0, 0.0, 1.0], [
                [-h, -h, h],
                [h, -h, h],
                [h, h, h],
                [-h, h, h],
            ]),
            ([0.0, 0.0, -1.0], [
                [h, -h, -h],
                [-h, -h, -h],
                [-h, h, -h],
                [h, h, -h],
            ]),
            ([1.0, 0.0, 0.0], [
                [h, -h, h],
                [h, -h, -h],
                [h, h, -h],
                [h, h, h],
            ]),
            ([-1.0, 0.0, 0.0], [
                [-h, -h, -h],
                [-h, -h, h],
                [-h, h, h],
                [-h, h, -h],
            ]),
            ([0.0, 1.0, 0.0], [
                [-h, h, h],
                [h, h, h],
                [h, h, -h],
                [-h, h, -h],
            ]),
            ([0.0, -1.0, 0.0], [
                [-h, -h, -h],
                [h, -h, -h],
                [h, -h, h],
                [-h, -h, h],
            ]),
        ];

        let uvs: [[f32; 2]; 4] = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        for (normal, corners) in faces {
            let base = vertices.len() as u32;

            for (corner, uv) in corners.into_iter().zip(uvs) {
                vertices.push(MeshVertex {
                    position: corner.into(),
                    normal: normal.into(),
                    uv: uv.into(),
                });
            }

            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }

        Self::new(vertices, indices)
    }
}

pub(crate) struct GeometryBuffers {
    pub(crate) vertex: gpu::Buffer<MeshVertex>,
    pub(crate) index: gpu::Buffer<u32>,
    pub(crate) version: u64,
    pub(crate) last_seen: u64,
}
