use crate::{Color, Renderer, Transform, gpu, mesh::RawMesh};
use common::{dirty::DirtyTracked, utils::Label};
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use macros::{Get, Set, With};
use std::{
    cell::{Cell, RefCell},
    hash::Hash,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Font {
    pub(crate) label: Label,
    inner: fontdue::Font,
    size: u8,
}

impl Font {
    pub(crate) fn new(label: Label, bytes: &[u8], size: u8) -> Self {
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
            .expect("Failed to load font");

        Self {
            label,
            inner: font,
            size,
        }
    }

    #[inline]
    pub(crate) fn rasterize(&self, ch: char) -> (fontdue::Metrics, Vec<u8>) {
        self.inner.rasterize(ch, self.size as f32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    font_hash: u32,
    char_code: u32,
}

impl GlyphKey {
    pub fn new(font: &Label, char_code: u32) -> Self {
        Self {
            font_hash: font.raw(),
            char_code,
        }
    }
}

#[derive(Get, Set, With)]
pub struct Text {
    pub font: Label,
    layout: RefCell<Layout>,

    #[get]
    #[set(into)]
    #[with]
    pub content: DirtyTracked<String>,

    #[get]
    #[set(into)]
    #[with]
    pub color: DirtyTracked<Color>,

    #[with]
    pub transform: DirtyTracked<Transform>,

    cached_instances: RefCell<Vec<RawMesh>>,
    cache_valid: Cell<bool>,
}

impl Deref for Text {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}

impl Text {
    pub fn new(font: Label, content: impl Into<String>) -> Self {
        Self {
            font,
            content: DirtyTracked::new(content.into()),
            layout: RefCell::new(Layout::new(CoordinateSystem::PositiveYDown)),
            color: Color::White.into(),
            transform: Transform::default().into(),
            cached_instances: RefCell::new(Vec::new()),
            cache_valid: Cell::new(false),
        }
    }

    #[inline]
    pub fn invalidate_cache(&self) {
        self.cache_valid.set(false)
    }

    #[inline]
    pub(crate) fn get_cached_instances(&self) -> std::cell::Ref<'_, Vec<RawMesh>> {
        // Rebuild cache if content changed
        if self.content.is_dirty() {
            self.cache_valid.set(false);
            self.content.clean();

            // Recompute layout
            let mut layout = self.layout.borrow_mut();
            let guard = gpu().fonts.load();
            let font = guard.get(&self.font).expect("Failed to get font");
            layout.clear();
            layout.append(
                &[&font.inner],
                &TextStyle::new(&self.content, font.size as f32, 0),
            );
        }

        if !self.cache_valid.get() {
            self.rebuild_cache();
        }

        self.cached_instances.borrow()
    }

    fn rebuild_cache(&self) {
        let layout = self.layout.borrow();
        let glyphs = layout.glyphs();
        let atlas_guard = gpu().texture_atlas.load();

        let mut instances = self.cached_instances.borrow_mut();
        instances.clear();
        instances.reserve(glyphs.len());

        for glyph in glyphs {
            let key = GlyphKey::new(&self.font, glyph.parent as u32);

            if let Some(uv) = atlas_guard.get_glyph_uv_coords(&key) {
                instances.push(RawMesh {
                    position: [
                        self.position.x + glyph.x * self.scale.x,
                        self.position.y + glyph.y * self.scale.y,
                        0.0,
                    ],
                    scale: [
                        glyph.width as f32 * self.scale.x,
                        glyph.height as f32 * self.scale.y,
                        1.0,
                    ],
                    rotation: [0.0, 0.0, self.rotation],
                    color: (*self.color).into(),
                    uv_offset: [uv.min_x, uv.min_y],
                    uv_scale: [uv.max_x - uv.min_x, uv.max_y - uv.min_y],
                });
            }
        }

        self.cache_valid.set(true);
    }

    #[inline]
    pub(crate) fn is_dirty(&self) -> bool {
        self.transform.is_dirty() || self.color.is_dirty()
    }

    #[inline]
    pub(crate) fn clean(&self) {
        self.transform.clean();
        self.color.clean();
    }

    #[inline]
    pub fn render(&self, renderer: &mut Renderer) {
        renderer.draw_text(self);
    }
}
