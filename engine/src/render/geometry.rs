use gpu::Gpu;
use gpu::LayoutDesc;
use utils::Handle;
use utils::SlotMap;

use crate::render::vertex::MeshVertex;

struct GeometryBuffers {
    vertices: gpu::Buffer<MeshVertex>,
    indices: gpu::Buffer<u32>,
}

pub struct Geometry {
    label: String,
    vertices: Vec<MeshVertex>,
    indices: Vec<u32>,
    buffers: Option<GeometryBuffers>,
    dirty: bool,
    pub layout: gpu::VertexLayout,
}

impl Geometry {
    pub fn new<L>(label: L, vertices: Vec<MeshVertex>, indices: Vec<u32>) -> Self
    where
        L: AsRef<str>,
    {
        Self {
            label: label.as_ref().to_owned(),
            vertices,
            indices,
            buffers: None,
            dirty: true,
            layout: MeshVertex::desc(),
        }
    }

    pub fn set(&mut self, vertices: Vec<MeshVertex>, indices: Vec<u32>) {
        self.vertices = vertices;
        self.indices = indices;
        self.dirty = true;
    }

    pub fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub(crate) fn bindings(&self) -> Option<(gpu::BufferBinding, gpu::BufferBinding)> {
        self.buffers
            .as_ref()
            .map(|b| (b.vertices.binding(), b.indices.binding()))
    }

    fn upload(&mut self, gpu: &Gpu, copy_pass: &gpu::CopyPass) {
        if !self.dirty || self.indices.is_empty() {
            return;
        }

        let buffers = self.buffers.get_or_insert_with(|| GeometryBuffers {
            vertices: gpu::Buffer::new(
                gpu,
                format!("{}-vertices", self.label),
                gpu::BufferUsages::VERTEX,
                self.vertices.len(),
            ),
            indices: gpu::Buffer::new(
                gpu,
                format!("{}-indices", self.label),
                gpu::BufferUsages::INDEX,
                self.indices.len(),
            ),
        });

        buffers.vertices.write_all(gpu, copy_pass, &self.vertices);
        buffers.indices.write_all(gpu, copy_pass, &self.indices);

        self.dirty = false;
    }
}

#[derive(Default)]
pub struct GeometryRegistry {
    geometries: SlotMap<Geometry>,
}

impl GeometryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, geometry: Geometry) -> Handle<Geometry> {
        self.geometries.insert(geometry)
    }

    pub fn get(&self, handle: Handle<Geometry>) -> Option<&Geometry> {
        self.geometries.get(handle)
    }

    pub fn get_mut(&mut self, handle: Handle<Geometry>) -> Option<&mut Geometry> {
        self.geometries.get_mut(handle)
    }

    pub(crate) fn upload_dirty(&mut self, gpu: &Gpu, copy_pass: &gpu::CopyPass) {
        for geometry in self.geometries.values_mut() {
            geometry.upload(gpu, copy_pass);
        }
    }

    pub fn len(&self) -> usize {
        self.geometries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.geometries.is_empty()
    }
}

pub fn cube() -> (Vec<MeshVertex>, Vec<u32>) {
    const FACES: [([f32; 3], [[f32; 3]; 4]); 6] = [
        // +z
        (
            [0.0, 0.0, 1.0],
            [
                [-0.5, -0.5, 0.5],
                [0.5, -0.5, 0.5],
                [0.5, 0.5, 0.5],
                [-0.5, 0.5, 0.5],
            ],
        ),
        // -z
        (
            [0.0, 0.0, -1.0],
            [
                [0.5, -0.5, -0.5],
                [-0.5, -0.5, -0.5],
                [-0.5, 0.5, -0.5],
                [0.5, 0.5, -0.5],
            ],
        ),
        // +x
        (
            [1.0, 0.0, 0.0],
            [
                [0.5, -0.5, 0.5],
                [0.5, -0.5, -0.5],
                [0.5, 0.5, -0.5],
                [0.5, 0.5, 0.5],
            ],
        ),
        // -x
        (
            [-1.0, 0.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [-0.5, -0.5, 0.5],
                [-0.5, 0.5, 0.5],
                [-0.5, 0.5, -0.5],
            ],
        ),
        // +y
        (
            [0.0, 1.0, 0.0],
            [
                [-0.5, 0.5, 0.5],
                [0.5, 0.5, 0.5],
                [0.5, 0.5, -0.5],
                [-0.5, 0.5, -0.5],
            ],
        ),
        // -y
        (
            [0.0, -1.0, 0.0],
            [
                [-0.5, -0.5, -0.5],
                [0.5, -0.5, -0.5],
                [0.5, -0.5, 0.5],
                [-0.5, -0.5, 0.5],
            ],
        ),
    ];

    const UVS: [[f32; 2]; 4] = [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];

    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    for (normal, corners) in FACES {
        let base = vertices.len() as u32;

        for (corner, uv) in corners.iter().zip(UVS) {
            vertices.push(MeshVertex {
                position: math::Vector3::new(corner[0], corner[1], corner[2]),
                normal: math::Vector3::new(normal[0], normal[1], normal[2]),
                uv: math::Vector2::new(uv[0], uv[1]),
            });
        }

        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    (vertices, indices)
}
