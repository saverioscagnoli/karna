use crate::retained::mesh::{GeometryBuffer, Mesh, MeshGpu};
use globals::consts;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use std::sync::Arc;
use utils::Handle;

pub struct Batch {
    pub buffer: Arc<GeometryBuffer>,
    pub handles: Vec<Handle<Mesh>>,
    pub instance_buffer: GpuBuffer<MeshGpu>,
    pub needs_rebuild: bool,
}

impl Batch {
    pub fn new(buffer: Arc<GeometryBuffer>) -> Self {
        Self {
            buffer,
            handles: Vec::new(),
            instance_buffer: GpuBufferBuilder::new()
                .label("Batch Instance Buffer")
                .vertex()
                .copy_dst()
                .capacity(consts::MESH_INSANCE_BASE_CAPACITY)
                .build(),
            needs_rebuild: false,
        }
    }
}
