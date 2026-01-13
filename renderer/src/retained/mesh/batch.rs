use crate::retained::mesh::{GeometryBuffer, Mesh, MeshGpu};
use globals::consts;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use std::sync::Arc;
use utils::Handle;

pub struct MeshBatch {
    pub buffer: Arc<GeometryBuffer>,
    pub handles: Vec<Handle<Mesh>>,
    pub instance_buffer: GpuBuffer<MeshGpu>,
    pub needs_rebuild: bool,
    capacity: usize,
}

impl MeshBatch {
    pub fn new(buffer: Arc<GeometryBuffer>) -> Self {
        Self {
            buffer,
            handles: Vec::new(),
            instance_buffer: GpuBufferBuilder::new()
                .label("Batch Instance Buffer")
                .vertex()
                .copy_dst()
                .capacity(consts::MESH_INSTANCE_BASE_CAPACITY)
                .build(),
            needs_rebuild: false,
            capacity: consts::MESH_INSTANCE_BASE_CAPACITY,
        }
    }

    /// Ensures the instance buffer can hold at least `required` instances.
    /// Returns true if the buffer was reallocated.
    pub fn ensure_capacity(&mut self, required: usize) -> bool {
        if required <= self.capacity {
            return false;
        }

        // Grow by 2x or to required size, whichever is larger
        let new_capacity = self.capacity.max(1).max(required).next_power_of_two();

        self.instance_buffer = GpuBufferBuilder::new()
            .label("Batch Instance Buffer")
            .vertex()
            .copy_dst()
            .capacity(new_capacity)
            .build();

        self.capacity = new_capacity;
        true
    }
}
