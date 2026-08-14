use std::rc::Rc;

use crate::BufferUsage;
use crate::GpuBuffer;
use crate::gpu::Gpu;
use crate::render::vertex::Vertex;

pub struct ImmediateGeometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: GpuBuffer<Vertex>,
    pub index_buffer: GpuBuffer<u32>,
}

impl ImmediateGeometry {
    pub fn new(gpu: Rc<Gpu>, label: &str, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let vertex_buffer = GpuBuffer::new(gpu.clone(), label, 1024, BufferUsage::VERTEX);
        let index_buffer = GpuBuffer::new(gpu, label, 4096, BufferUsage::INDEX);

        Self {
            vertices,
            indices,
            vertex_buffer,
            index_buffer,
        }
    }
}
