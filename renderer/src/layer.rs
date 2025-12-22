use std::sync::Arc;

use assets::AssetManager;
use math::Vector2;
use utils::label;
use wgpu::naga::FastHashMap;

use crate::{Text, camera::Camera, text::TextRenderer};

pub struct RenderLayer {
    assets: Arc<AssetManager>,
    pub camera: Camera,
    text_renderer: TextRenderer,

    /// All the texts that are / were written with `draw_debug_text`
    debug_texts: FastHashMap<String, Text>,

    /// The texts that are currently being used
    debug_texts_used: Vec<String>,
}

impl RenderLayer {
    #[inline]
    pub fn new(assets: Arc<AssetManager>, camera: Camera, text_renderer: TextRenderer) -> Self {
        Self {
            assets,
            camera,
            text_renderer,
            debug_texts: FastHashMap::default(),
            debug_texts_used: Vec::new(),
        }
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
    pub fn draw_text(&mut self, text: &mut Text) {
        text.rebuild(&self.assets);

        for glyph in text.glyph_instances() {
            self.text_renderer.add_glyph(*glyph);
        }
    }

    #[inline]
    pub fn draw_debug_text<T: Into<String>, P: Into<Vector2>>(&mut self, text: T, pos: P) {
        let content = text.into();
        let pos = pos.into();

        let key = format!("{}:{}", pos.x.round() as i32, pos.y.round() as i32);

        let text = self
            .debug_texts
            .entry(key.clone())
            .or_insert_with(|| Text::new(label!("debug")));

        // Trigger dirty
        if text.content() != content {
            text.set_content(content);
        }

        if text.position() != &pos {
            text.set_position(pos);
        }

        text.rebuild(&self.assets);

        for glyph in text.glyph_instances() {
            self.text_renderer.add_glyph(*glyph);
        }

        self.debug_texts_used.push(key);
    }

    #[inline]
    pub(crate) fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.text_renderer
            .render(render_pass, &self.camera, &self.assets);
    }

    #[inline]
    pub(crate) fn clear(&mut self) {
        self.text_renderer.clear();
        self.camera.clean();

        self.debug_texts
            .retain(|key, _| self.debug_texts_used.contains(key));
        self.debug_texts_used.clear();
    }
}
