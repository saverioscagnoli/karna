use crate::{Color, Renderer, Transform, gpu};
use common::{dirty::DirtyTracked, utils::Label};
use fontdue::layout::{CoordinateSystem, GlyphPosition, Layout, TextStyle};
use macros::{Get, Set, With};
use std::{
    cell::{Ref, RefCell},
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
    pub color: Color,

    #[with]
    pub transform: Transform,
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
            color: Color::White,
            transform: Transform::default(),
        }
    }

    #[inline]
    pub(crate) fn compute_glyphs(&self) -> Ref<'_, [GlyphPosition]> {
        if self.content.is_dirty() {
            let mut layout = self.layout.borrow_mut();
            let guard = gpu().fonts.load();
            let font = guard.get(&self.font).expect("Failed to get font");

            layout.clear();
            layout.append(
                &[&font.inner],
                &TextStyle::new(&self.content, font.size as f32, 0),
            );

            self.content.clean();
        }

        Ref::map(self.layout.borrow(), |layout| layout.glyphs().as_slice())
    }

    #[inline]
    pub fn render(&self, renderer: &mut Renderer) {
        renderer.draw_text(self);
    }
}
