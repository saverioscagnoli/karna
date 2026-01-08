mod handle;
mod text;

pub mod mesh;

use crate::{
    Camera,
    retained::mesh::{Mesh, MeshBatch, MeshGpu},
    retained_shader,
    traits::LayoutDescriptor,
    vertex::Vertex,
};
use assets::AssetServer;
use globals::{consts, profiling};
use logging::warn;
use utils::{FastHashMap, Handle, SlotMap};

pub use handle::*;
pub use text::*;

pub struct RetainedRenderer {
    meshes: SlotMap<Mesh>,
    batches: FastHashMap<u64, MeshBatch>,
    mesh_to_batch: FastHashMap<u64, u64>, // handle hash -> geometry_id

    pipeline: wgpu::RenderPipeline,
}

impl RetainedRenderer {
    #[doc(hidden)]
    pub fn new(surface_format: wgpu::TextureFormat, camera: &Camera, assets: &AssetServer) -> Self {
        let pipeline = retained_shader()
            .pipeline_builder()
            .label("Retained Triangle Pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .cull_mode(wgpu::Face::Front)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), assets.atlas_bgl()],
                &[Vertex::desc(), MeshGpu::desc()],
            );

        Self {
            meshes: SlotMap::with_capacity(consts::MESH_INSTANCE_BASE_CAPACITY),
            batches: FastHashMap::default(),
            mesh_to_batch: FastHashMap::default(),
            pipeline,
        }
    }

    #[inline]
    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        let geometry_id = mesh.geometry().id;
        let buffer = mesh.geometry().buffer.clone();

        let batch = self
            .batches
            .entry(geometry_id)
            .or_insert_with(|| MeshBatch::new(buffer));

        let handle = self.meshes.insert(mesh);

        batch.handles.push(handle);
        batch.needs_rebuild = true;

        self.mesh_to_batch
            .insert(Self::handle_key(handle), geometry_id);

        handle
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
        let key = Self::handle_key(handle);

        if let Some(geometry_id) = self.mesh_to_batch.remove(&key) {
            if let Some(batch) = self.batches.get_mut(&geometry_id) {
                batch.handles.retain(|&h| h != handle);
                batch.needs_rebuild = true;

                if batch.handles.is_empty() {
                    self.batches.remove(&geometry_id);
                }
            }
        }

        self.meshes.remove(handle);
    }

    #[inline]
    fn handle_key(handle: Handle<Mesh>) -> u64 {
        // Combine index and generation into a single u64 key
        ((handle.index() as u64) << 32) | (handle.generation() as u64)
    }

    pub(crate) fn present<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        assets: &AssetServer,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        profiling::record_pipeline_switches(1);

        let mut writes = 0;

        for batch in self.batches.values_mut() {
            if batch.handles.is_empty() {
                continue;
            }

            if batch.needs_rebuild {
                // Full rebuild: collect all instance data
                warn!("Rebuilding instance buffer");
                let instance_data: Vec<MeshGpu> = batch
                    .handles
                    .iter()
                    .filter_map(|&h| {
                        let mesh = self.meshes.get_mut(h)?;
                        mesh.prepare(assets);
                        Some(mesh.gpu)
                    })
                    .collect();

                batch.instance_buffer.write_from_index(0, &instance_data);
                batch.needs_rebuild = false;

                writes = instance_data.len() as u32;
            } else {
                for (buffer_idx, &handle) in batch.handles.iter().enumerate() {
                    if let Some(mesh) = self.meshes.get_mut(handle)
                        && mesh.prepare(assets)
                    {
                        batch
                            .instance_buffer
                            .write_from_index(buffer_idx, &[mesh.gpu]);

                        writes += 1;
                    }
                }
            }
        }

        profiling::record_instance_writes(writes);

        for batch in self.batches.values() {
            if batch.handles.is_empty() {
                continue;
            }

            let vertex_count = batch.buffer.vertex_buffer.len() as u32;
            let index_count = batch.buffer.index_buffer.len() as u32;

            render_pass.set_vertex_buffer(0, batch.buffer.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, batch.instance_buffer.slice(..));
            render_pass.set_index_buffer(
                batch.buffer.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            render_pass.draw_indexed(0..index_count, 0, 0..batch.handles.len() as u32);
            profiling::record_draw_call(vertex_count, index_count);
        }
    }
}
