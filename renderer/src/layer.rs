use std::sync::Arc;

use crate::{
    Sprite,
    camera::Camera,
    immediate::ImmediateRenderer,
    mesh::{Mesh, MeshInstanceGpu},
    text::{Text, TextRenderer2d},
};
use assets::AssetManager;
use gpu::core::{GpuBuffer, GpuBufferBuilder};
use utils::{Handle, SlotMap};
use wgpu::naga::FastHashMap;

#[derive(Debug, Clone, Copy)]
pub enum Layer {
    World,
    Ui,
    N(usize),
}

pub struct RenderLayer {
    pub camera: Camera,
    assets: Arc<AssetManager>,

    // Mesh rendering
    meshes: SlotMap<Mesh>,
    instance_buffer: GpuBuffer<MeshInstanceGpu>,
    instance_capacity: usize,

    pub(crate) immediate: ImmediateRenderer,

    // Text rendering
    texts: SlotMap<Text>,
    text_renderer: TextRenderer2d,

    sprites: SlotMap<Sprite>,

    // Reusable buffers
    batches: FastHashMap<u32, Vec<MeshInstanceGpu>>,
    all_instances: Vec<MeshInstanceGpu>,
    batch_ranges: Vec<(u32, std::ops::Range<u32>)>,
    any_dirty: bool,
}

impl RenderLayer {
    pub(crate) fn new(
        surface_format: wgpu::TextureFormat,
        camera: Camera,
        assets: Arc<AssetManager>,
    ) -> Self {
        let instance_buffer = GpuBufferBuilder::new()
            .label("render layer instance buffer")
            .vertex()
            .copy_dst()
            .capacity(Mesh::INITIAL_INSTANCE_CAPACITY)
            .build();

        let text_renderer = TextRenderer2d::new("world", surface_format, &camera, &assets);

        Self {
            camera,
            assets,
            meshes: SlotMap::new(),
            instance_buffer,
            instance_capacity: Mesh::INITIAL_INSTANCE_CAPACITY,
            sprites: SlotMap::new(),
            immediate: ImmediateRenderer::new(),
            texts: SlotMap::new(),
            text_renderer,
            batches: FastHashMap::default(),
            all_instances: Vec::new(),
            batch_ranges: Vec::new(),
            any_dirty: false,
        }
    }

    // Mesh methods (existing)
    #[inline]
    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.any_dirty = true;
        self.meshes.insert(mesh)
    }

    #[inline]
    pub fn get_mesh(&self, id: Handle<Mesh>) -> &Mesh {
        self.meshes.get(id).expect("Failed to get mesh instance")
    }

    #[inline]
    pub fn get_mesh_mut(&mut self, id: Handle<Mesh>) -> &mut Mesh {
        self.meshes
            .get_mut(id)
            .expect("Failed to get mesh instance")
    }

    #[inline]
    pub fn remove_mesh(&mut self, id: Handle<Mesh>) {
        self.any_dirty = true;
        self.meshes.remove(id);
    }

    // Text methods (new)
    #[inline]
    pub fn add_text(&mut self, text: Text) -> Handle<Text> {
        self.texts.insert(text)
    }

    #[inline]
    pub fn get_text(&self, id: Handle<Text>) -> &Text {
        self.texts.get(id).expect("Failed to get text instance")
    }

    #[inline]
    pub fn get_text_mut(&mut self, id: Handle<Text>) -> &mut Text {
        self.texts.get_mut(id).expect("Failed to get text instance")
    }

    #[inline]
    pub fn remove_text(&mut self, id: Handle<Text>) {
        self.texts.remove(id);
    }

    #[inline]
    pub fn add_sprite(&mut self, sprite: Sprite) -> Handle<Sprite> {
        self.sprites.insert(sprite)
    }

    #[inline]
    pub fn get_sprite(&self, id: Handle<Sprite>) -> &Sprite {
        self.sprites.get(id).expect("Failed to get sprite instance")
    }

    #[inline]
    pub fn get_sprite_mut(&mut self, id: Handle<Sprite>) -> &mut Sprite {
        self.sprites
            .get_mut(id)
            .expect("Failed to get sprite instance")
    }

    #[inline]
    pub fn remove_sprite(&mut self, id: Handle<Sprite>) {
        self.sprites.remove(id);
    }

    #[inline]
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    #[inline]
    pub(crate) fn update(&mut self, width: u32, height: u32, dt: f32) {
        if self.camera.dirty() {
            self.camera.resize(width, height);
        }
        self.camera.update_shake(dt);
    }

    #[inline]
    pub(crate) fn present<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        retained_pipeline: &'a wgpu::RenderPipeline,
        immediate_pipeline: &'a wgpu::RenderPipeline,
    ) {
        render_pass.set_pipeline(retained_pipeline);
        render_pass.set_bind_group(0, self.camera.view_projection_bind_group(), &[]);
        render_pass.set_bind_group(1, self.assets.bind_group(), &[]);

        let mut has_dirty = self.any_dirty;

        // Batch regular meshes
        for mesh in self.meshes.values_mut() {
            if !mesh.visible() {
                continue;
            }

            if mesh.is_dirty() {
                mesh.sync_gpu(&self.assets);
                has_dirty = true;
            }

            let geo_id = mesh.geometry().id();
            self.batches
                .entry(geo_id)
                .or_insert_with(Vec::new)
                .push(mesh.gpu());
        }

        // Batch sprites (they deref to Mesh, so we can access their mesh data)
        for sprite in self.sprites.values_mut() {
            if !sprite.visible() {
                continue;
            }

            if sprite.is_dirty() {
                sprite.sync_gpu(&self.assets);
                has_dirty = true;
            }

            let geo_id = sprite.geometry().id();
            self.batches
                .entry(geo_id)
                .or_insert_with(Vec::new)
                .push(sprite.gpu());
        }

        if !self.batches.is_empty() {
            if has_dirty {
                let total_instances: usize = self.batches.values().map(|v| v.len()).sum();

                if total_instances > self.instance_capacity {
                    self.instance_capacity = (total_instances * 2).max(128);
                    self.instance_buffer.resize(self.instance_capacity);
                }

                self.all_instances.clear();
                self.batch_ranges.clear();

                for (geo_id, instances) in self.batches.iter() {
                    let start = self.all_instances.len() as u32;
                    self.all_instances.extend_from_slice(instances);
                    let end = self.all_instances.len() as u32;
                    self.batch_ranges.push((*geo_id, start..end));
                }

                self.instance_buffer.write(0, &self.all_instances);
                self.any_dirty = false;
            } else {
                self.batch_ranges.clear();
                let mut offset = 0u32;

                for (geo_id, instances) in self.batches.iter() {
                    let start = offset;
                    let end = offset + instances.len() as u32;
                    self.batch_ranges.push((*geo_id, start..end));
                    offset = end;
                }
            }

            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));

            let mut geo_buffers: Vec<(u32, Arc<crate::mesh::GeometryBuffer>)> = Vec::new();

            for geo_id in self.batches.keys() {
                let geo_buffer = self
                    .meshes
                    .values()
                    .find(|m| m.geometry().id() == *geo_id)
                    .map(|m| m.geometry().buffer())
                    .or_else(|| {
                        self.sprites
                            .values()
                            .find(|s| s.geometry().id() == *geo_id)
                            .map(|s| s.geometry().buffer())
                    })
                    .expect("Geometry should exist");

                geo_buffers.push((*geo_id, geo_buffer));
            }

            for batch in self.batches.values_mut() {
                batch.clear();
            }

            for (geo_id, instance_range) in &self.batch_ranges {
                let geo_buffer = geo_buffers
                    .iter()
                    .find(|(id, _)| id == geo_id)
                    .map(|(_, buf)| buf)
                    .expect("Geometry buffer should exist");

                render_pass.set_vertex_buffer(0, geo_buffer.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(geo_buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(
                    0..geo_buffer.index_count as u32,
                    0,
                    instance_range.clone(),
                );
            }
        }

        self.text_renderer.clear();

        for text in self.texts.values_mut() {
            text.rebuild(&self.assets);

            for glyph in text.glyph_instances() {
                self.text_renderer.add_glyph(*glyph);
            }
        }

        self.text_renderer
            .present(render_pass, &self.camera, &self.assets);

        // Present immediate rendering
        render_pass.set_pipeline(immediate_pipeline);
        render_pass.set_bind_group(0, self.camera.view_projection_bind_group(), &[]);
        render_pass.set_bind_group(1, self.assets.bind_group(), &[]);

        self.immediate.present(render_pass);
    }
}
