use crate::{Color, GPU, Renderer, Transform};
use common::utils::Label;
use fontdue::layout::{CoordinateSystem, GlyphPosition, Layout, TextStyle};

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

pub struct Text {
    pub font: Label,
    pub layout: Layout,
    pub content: String,
    pub color: Color,
    pub transform: Transform,
}

impl std::ops::Deref for Text {
    type Target = Transform;

    fn deref(&self) -> &Self::Target {
        &self.transform
    }
}

impl std::ops::DerefMut for Text {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.transform
    }
}

impl Text {
    pub fn new(font: Label, content: impl Into<String>) -> Self {
        Self {
            font,
            content: content.into(),
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            color: Color::White,
            transform: Transform::default(),
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    #[inline]
    pub fn render(&mut self, gpu: &GPU, renderer: &mut Renderer) {
        let lock = gpu.fonts.read().expect("Fonts lock is poisoned");
        let font = lock.get(&self.font).expect("Failed to get font");

        self.layout.append(
            &[&font.inner],
            &TextStyle::new(&self.content, font.size as f32, 0),
        );

        renderer.draw_text(self);

        self.layout.clear()
    }
}
