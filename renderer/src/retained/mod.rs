pub mod mesh;

mod handle;

use crate::retained::mesh::{GeometryBuffer, Mesh, MeshGpu};
use globals::consts;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use std::sync::Arc;
use utils::{FastHashMap, Handle, SlotMap};

pub use handle::*;

pub struct RetainedRenderer {
    geometry_cache: FastHashMap<u64, Arc<GeometryBuffer>>,
    meshes: SlotMap<Mesh>,
    instance_buffer: GpuBuffer<MeshGpu>,
}

impl RetainedRenderer {
    #[doc(hidden)]
    pub fn new() -> Self {
        let instance_buffer = GpuBufferBuilder::new()
            .label("Mesh instance buffer")
            .vertex()
            .copy_dst()
            .capacity(consts::MESH_INSANCE_BASE_CAPACITY)
            .build();

        Self {
            geometry_cache: FastHashMap::default(),
            meshes: SlotMap::with_capacity(consts::MESH_INSANCE_BASE_CAPACITY),
            instance_buffer,
        }
    }

    #[inline]
    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.meshes.insert(mesh)
    }

    #[inline]
    pub fn get_mesh(&self, handle: Handle<Mesh>) -> Option<&Mesh> {
        self.meshes.get(handle)
    }

    #[inline]
    pub fn get_mesh_mut(&mut self, handle: Handle<Mesh>) -> Option<&mut Mesh> {
        self.meshes.get_mut(handle)
    }

    #[inline]
    pub fn remove_mesh(&mut self, handle: Handle<Mesh>) {
        self.meshes.remove(handle);
    }
}
