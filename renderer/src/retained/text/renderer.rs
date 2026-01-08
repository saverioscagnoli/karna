use crate::{
    Camera,
    retained::{
        GlyphGpu, Text,
        mesh::{Geometry, GeometryBuffer},
        text::batch::TextBatch,
    },
    text_shader,
    traits::LayoutDescriptor,
    vertex::Vertex,
};
use assets::AssetServer;
use globals::profiling;
use std::sync::Arc;
use utils::{FastHashMap, Handle, SlotMap};

pub struct TextRenderer {
    texts: SlotMap<Text>,
    batches: FastHashMap<u64, TextBatch>, // font handle -> batch
    text_to_font: FastHashMap<u64, u64>,  // text handle hash -> font handle hash

    quad_geometry: Arc<GeometryBuffer>,
    pipeline: wgpu::RenderPipeline,
}

impl TextRenderer {
    pub fn new(surface_format: wgpu::TextureFormat, camera: &Camera, assets: &AssetServer) -> Self {
        let quad_geometry = Geometry::unit_rect();

        let pipeline = text_shader()
            .pipeline_builder()
            .label("Text Pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), assets.atlas_bgl()],
                &[Vertex::desc(), GlyphGpu::desc()],
            );

        Self {
            texts: SlotMap::with_capacity(256),
            batches: FastHashMap::default(),
            text_to_font: FastHashMap::default(),
            quad_geometry: quad_geometry.buffer,
            pipeline,
        }
    }

    #[inline]
    pub fn add_text(&mut self, text: Text) -> Handle<Text> {
        let font_key = text.font().index() as u64;

        let batch = self
            .batches
            .entry(font_key)
            .or_insert_with(|| TextBatch::new());

        let handle = self.texts.insert(text);

        batch.handles.push(handle);
        batch.needs_rebuild = true;

        self.text_to_font.insert(Self::handle_key(handle), font_key);

        handle
    }

    #[inline]
    pub fn get_text(&self, handle: Handle<Text>) -> Option<&Text> {
        self.texts.get(handle)
    }

    #[inline]
    pub fn get_text_mut(&mut self, handle: Handle<Text>) -> Option<&mut Text> {
        if let Some(font_key) = self.text_to_font.get(&Self::handle_key(handle)) {
            if let Some(batch) = self.batches.get_mut(font_key) {
                batch.needs_rebuild = true;
            }
        }

        self.texts.get_mut(handle)
    }

    #[inline]
    pub fn remove_text(&mut self, handle: Handle<Text>) {
        let key = Self::handle_key(handle);

        if let Some(font_key) = self.text_to_font.remove(&key) {
            if let Some(batch) = self.batches.get_mut(&font_key) {
                batch.handles.retain(|&h| h != handle);
                batch.needs_rebuild = true;

                if batch.handles.is_empty() {
                    self.batches.remove(&font_key);
                }
            }
        }

        self.texts.remove(handle);
    }

    #[inline]
    fn handle_key(handle: Handle<Text>) -> u64 {
        ((handle.index() as u64) << 32) | (handle.generation() as u64)
    }

    #[inline]
    pub(crate) fn present<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        assets: &AssetServer,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        profiling::record_pipeline_switches(1);

        for batch in self.batches.values_mut() {
            if batch.handles.is_empty() {
                continue;
            }

            if batch.needs_rebuild {
                let mut all_glyphs: Vec<GlyphGpu> = Vec::new();

                for &handle in &batch.handles {
                    if let Some(text) = self.texts.get_mut(handle) {
                        text.prepare(assets);

                        all_glyphs.extend_from_slice(&text.gpu_glyphs);
                    }
                }

                if !all_glyphs.is_empty() {
                    batch.instance_buffer.write_from_index(0, &all_glyphs);
                }

                batch.total_glyphs = all_glyphs.len();
                batch.needs_rebuild = false;

                profiling::record_instance_writes(all_glyphs.len() as u32);
            }
        }

        for batch in self.batches.values() {
            if batch.total_glyphs == 0 {
                continue;
            }

            render_pass.set_vertex_buffer(0, self.quad_geometry.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, batch.instance_buffer.slice(..));
            render_pass.set_index_buffer(
                self.quad_geometry.index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );

            let index_count = self.quad_geometry.index_buffer.len() as u32;
            render_pass.draw_indexed(0..index_count, 0, 0..batch.total_glyphs as u32);

            profiling::record_draw_call(4, index_count);
        }
    }
}
