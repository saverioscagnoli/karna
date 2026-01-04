mod text;
mod text_renderer;

use crate::{Mesh, Sprite, mesh::MeshInstanceGpu, retained::text_renderer::RetainedTextRenderer};
use assets::AssetManager;
use globals::profiling;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use std::sync::Arc;
use utils::{Handle, SlotMap};
use wgpu::naga::FastHashMap;

pub use text::*;
pub use text_renderer::{GlyphInstance, TextVertex};

pub struct RetainedRenderer {
    assets: Arc<AssetManager>,
    meshes: SlotMap<Mesh>,
    sprites: SlotMap<Sprite>,
    text_renderer: RetainedTextRenderer,
    instance_buffer: GpuBuffer<MeshInstanceGpu>,
    dirty_batches: Vec<(usize, MeshInstanceGpu)>,
    geometry_groups: FastHashMap<u32, Vec<usize>>,
}

impl RetainedRenderer {
    pub fn new(assets: Arc<AssetManager>) -> Self {
        let instance_buffer = GpuBufferBuilder::new()
            .label("render layer instance buffer")
            .vertex()
            .copy_dst()
            .capacity(Mesh::INITIAL_INSTANCE_CAPACITY)
            .build();

        let text_renderer = RetainedTextRenderer::new();

        Self {
            assets,
            meshes: SlotMap::new(),
            sprites: SlotMap::new(),
            text_renderer,
            instance_buffer,
            dirty_batches: Vec::new(),
            geometry_groups: FastHashMap::default(),
        }
    }

    #[inline]
    pub(crate) fn add_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.meshes.insert(mesh)
    }

    #[inline]
    pub(crate) fn get_mesh(&self, handle: Handle<Mesh>) -> &Mesh {
        self.meshes
            .get(handle)
            .expect("Failed to get mesh instance")
    }

    #[inline]
    pub(crate) fn get_mesh_mut(&mut self, handle: Handle<Mesh>) -> &mut Mesh {
        self.meshes
            .get_mut(handle)
            .expect("Failed to get mutable mesh instance")
    }

    #[inline]
    pub(crate) fn remove_mesh(&mut self, handle: Handle<Mesh>) {
        self.meshes.remove(handle);
    }

    // === Text ===

    #[inline]
    pub(crate) fn add_text(&mut self, text: Text) -> Handle<Text> {
        self.text_renderer.add_text(text)
    }

    #[inline]
    pub(crate) fn get_text(&self, handle: Handle<Text>) -> &Text {
        self.text_renderer.get_text(handle)
    }

    #[inline]
    pub(crate) fn get_text_mut(&mut self, handle: Handle<Text>) -> &mut Text {
        self.text_renderer.get_text_mut(handle)
    }

    #[inline]
    pub(crate) fn remove_text(&mut self, handle: Handle<Text>) {
        self.text_renderer.remove_text(handle);
    }

    // === Sprite ===

    #[inline]
    pub(crate) fn add_sprite(&mut self, sprite: Sprite) -> Handle<Sprite> {
        self.sprites.insert(sprite)
    }

    #[inline]
    pub(crate) fn get_sprite(&self, handle: Handle<Sprite>) -> &Sprite {
        self.sprites
            .get(handle)
            .expect("Failed to get sprite instance")
    }

    #[inline]
    pub(crate) fn get_sprite_mut(&mut self, handle: Handle<Sprite>) -> &mut Sprite {
        self.sprites
            .get_mut(handle)
            .expect("Failed to get mutable sprite instance")
    }

    #[inline]
    pub(crate) fn remove_sprite(&mut self, handle: Handle<Sprite>) {
        self.sprites.remove(handle);
    }

    #[inline]
    pub(crate) fn present<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        pipeline: &'a wgpu::RenderPipeline,
        text_pipeline: &'a wgpu::RenderPipeline,
    ) {
        render_pass.set_pipeline(pipeline);
        profiling::record_pipeline_switches(1);

        for (idx, mesh) in self.meshes.values_mut().enumerate() {
            if mesh.sync_gpu(&self.assets) {
                self.dirty_batches.push((idx, mesh.gpu()));
            }
        }

        for (idx, instance) in &self.dirty_batches {
            let byte_offset = (idx * std::mem::size_of::<MeshInstanceGpu>()) as u64;
            self.instance_buffer.write(byte_offset, &[*instance]);
        }

        profiling::record_instance_writes(self.dirty_batches.len() as u32);

        self.dirty_batches.clear();

        for (idx, mesh) in self.meshes.values().enumerate() {
            self.geometry_groups
                .entry(mesh.geometry().id())
                .or_insert_with(Vec::new)
                .push(idx);
        }

        for indices in self.geometry_groups.values() {
            let Some(first) = indices.first() else {
                continue;
            };

            let mesh = self.meshes.values().nth(*first).unwrap();
            let geom_buffer = mesh.geometry().buffer();

            render_pass.set_vertex_buffer(0, geom_buffer.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(
                geom_buffer.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            let start = *indices.first().unwrap() as u32;
            let end = (*indices.last().unwrap() + 1) as u32;

            render_pass.draw_indexed(0..geom_buffer.index_count as u32, 0, start..end);

            let vertex_count = geom_buffer.vertex_count as u32;
            let index_count = geom_buffer.index_count as u32;
            let instance_count = end - start;

            profiling::record_draw_call(vertex_count, index_count);
            profiling::record_triangles(index_count * instance_count);
        }

        self.geometry_groups.clear();

        self.text_renderer.prepare(&self.assets);
        self.text_renderer.present(render_pass, text_pipeline);
    }
}
