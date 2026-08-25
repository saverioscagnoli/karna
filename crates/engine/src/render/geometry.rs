use std::rc::Rc;

use crate::BufferUsage;
use crate::GpuBuffer;
use crate::gpu::Gpu;
use crate::render::vertex::Vertex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchTexture {
    White,
    Atlas(usize),
}

#[derive(Debug, Clone, Copy)]
pub struct Batch {
    pub texture: BatchTexture,
    pub index_start: u32,
    pub index_count: u32,
}

pub struct ImmediateGeometry {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub batches: Vec<Batch>,
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
            batches: Vec::new(),
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn push_batch(&mut self, texture: BatchTexture, added: u32) {
        let start = self.indices.len() as u32 - added;

        if let Some(last) = self.batches.last_mut()
            && last.texture == texture
            && last.index_start + last.index_count == start
        {
            last.index_count += added;
            return;
        }

        self.batches.push(Batch {
            texture,
            index_start: start,
            index_count: added,
        });
    }
}
