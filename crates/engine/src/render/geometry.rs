use crate::gpu::BufferUsage;
use crate::gpu::Device;
use crate::gpu::GpuBuffer;
use crate::render::vertex::Vertex;

#[derive(Debug, Clone, Copy)]
pub struct Batch {
    pub page: usize,
    pub start: u32,
    pub count: u32,
}

pub struct ImmediateGeometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub batches: Vec<Batch>,
    pub vertex_buffer: GpuBuffer<Vertex>,
    pub index_buffer: GpuBuffer<u32>,
}

impl ImmediateGeometry {
    pub fn new(device: Device, label: &str, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let vertex_buffer = GpuBuffer::new(device.raw(), label, 1024, BufferUsage::VERTEX);
        let index_buffer = GpuBuffer::new(device.raw(), label, 4096, BufferUsage::INDEX);

        Self {
            vertices,
            indices,
            batches: Vec::new(),
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn push_quad(&mut self, page: usize, quad: [Vertex; 4]) {
        let base = self.vertices.len() as u32;
        let start = self.indices.len() as u32;

        self.vertices.extend_from_slice(&quad);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);

        match self.batches.last_mut() {
            Some(batch) if batch.page == page => batch.count += 6,
            _ => self.batches.push(Batch {
                page,
                start,
                count: 6,
            }),
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.batches.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}
