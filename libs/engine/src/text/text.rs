use core::cell::OnceCell;
use core::hash::Hash;
use core::hash::Hasher;
use core::mem;

use cosmic_text as ct;
use nostd::alloc::string::String;
use nostd::alloc::sync::Arc;
use nostd::alloc::vec::Vec;
use nostd::collections::FxHasher;
use nostd::collections::Handle;
use sdl3::render::Color;

use crate::assets::AssetServer;
use crate::text::Font;
use crate::text::PositionedGlyph;

#[derive(Debug, Clone, Copy)]
pub struct TextStyle {
    pub font: Option<Handle<Font>>,
    pub size: Option<f32>,
    pub line_height: f32,
    pub wrap: Option<f32>,
    pub align: ct::Align,
    pub bold: bool,
    pub italic: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: None,
            size: None,
            line_height: 1.25,
            wrap: None,
            align: ct::Align::Left,
            bold: false,
            italic: false,
        }
    }
}

impl TextStyle {
    pub fn new(font: Handle<Font>, size: f32) -> Self {
        Self {
            font: Some(font),
            size: Some(size),
            ..Self::default()
        }
    }

    #[inline]
    pub fn font(&self) -> Option<Handle<Font>> {
        self.font
    }

    #[inline]
    pub fn set_font(&mut self, font: Handle<Font>) {
        self.font = Some(font);
    }

    pub fn with_font(mut self, font: Handle<Font>) -> Self {
        self.font = Some(font);
        self
    }

    #[inline]
    pub fn size(&self) -> Option<f32> {
        self.size
    }

    #[inline]
    pub fn set_size(&mut self, size: f32) {
        self.size = Some(size);
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    #[inline]
    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    #[inline]
    pub fn set_line_height(&mut self, line_height: f32) {
        self.line_height = line_height;
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }

    #[inline]
    pub fn wrap(&self) -> Option<f32> {
        self.wrap
    }

    #[inline]
    pub fn set_wrap(&mut self, wrap: f32) {
        self.wrap = Some(wrap)
    }

    pub fn with_wrap(mut self, wrap: f32) -> Self {
        self.wrap = Some(wrap);
        self
    }

    #[inline]
    pub fn align(&self) -> ct::Align {
        self.align
    }

    #[inline]
    pub fn set_align(&mut self, align: ct::Align) {
        self.align = align;
    }

    pub fn with_align(mut self, align: ct::Align) -> Self {
        self.align = align;
        self
    }

    #[inline]
    pub fn bold(&self) -> bool {
        self.bold
    }

    #[inline]
    pub fn set_bold(&mut self, bold: bool) {
        self.bold = bold;
    }

    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    #[inline]
    pub fn italic(&self) -> bool {
        self.italic
    }

    #[inline]
    pub fn set_italic(&mut self, italic: bool) {
        self.italic = italic;
    }

    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }
}

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct TextSpan {
    pub text: String,
    pub color: Option<Color>,
    pub font: Option<Handle<Font>>,
    pub bold: bool,
    pub italic: bool,
    pub metadata: usize,
}

impl TextSpan {
    pub fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    pub fn with_color<C>(mut self, color: C) -> Self
    where
        C: Into<Color>,
    {
        self.color = Some(color.into());
        self
    }

    pub fn with_font(mut self, font: Handle<Font>) -> Self {
        self.font = Some(font);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    pub fn with_metadata(mut self, metadata: usize) -> Self {
        self.metadata = metadata;
        self
    }
}

#[derive(Default)]
#[derive(Clone)]
pub struct Text {
    spans: Vec<TextSpan>,
    style: TextStyle,
    layout: OnceCell<Arc<TextLayout>>,
}

impl Text {
    pub fn new<S>(text: S) -> Self
    where
        S: Into<String>,
    {
        Self::rich([TextSpan::new(text)])
    }

    pub fn rich<I>(spans: I) -> Self
    where
        I: IntoIterator<Item = TextSpan>,
    {
        Self {
            spans: spans.into_iter().collect(),
            style: TextStyle::default(),
            layout: OnceCell::new(),
        }
    }

    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub fn set<S>(&mut self, text: S)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref();

        if let [span] = self.spans.as_mut_slice() {
            if span.text == text {
                return;
            }

            span.text.clear();
            span.text.push_str(text);
        } else {
            self.spans.clear();
            self.spans.push(TextSpan::new(text));
        }

        self.layout.take();
    }

    pub fn spans(&self) -> &[TextSpan] {
        &self.spans
    }

    pub fn spans_mut(&mut self) -> &mut Vec<TextSpan> {
        self.layout.take();
        &mut self.spans
    }

    pub fn style(&self) -> &TextStyle {
        &self.style
    }

    pub fn style_mut(&mut self) -> &mut TextStyle {
        self.layout.take();
        &mut self.style
    }

    pub fn set_style(&mut self, style: TextStyle) {
        self.style = style;
        self.layout.take();
    }

    pub fn size(&self, assets: &AssetServer) -> math::Size<f32> {
        self.layout(assets).size
    }

    pub fn caret_x(&self, assets: &AssetServer, byte: usize) -> f32 {
        self.layout(assets).caret_x(byte)
    }

    pub fn byte_at_x(&self, assets: &AssetServer, x: f32) -> usize {
        self.layout(assets).byte_at_x(x)
    }

    pub(crate) fn layout(&self, assets: &AssetServer) -> &TextLayout {
        self.layout
            .get_or_init(|| assets.layout(&self.spans, &self.style))
    }
}

pub(crate) struct TextLayout {
    pub(crate) glyphs: Vec<PositionedGlyph>,
    pub(crate) size: math::Size<f32>,
}

impl TextLayout {
    #[inline]
    fn advance(&self, index: usize) -> f32 {
        match self.glyphs.get(index + 1) {
            Some(next) if next.line == self.glyphs[index].line => next.pen.x,
            _ => self.size.w(),
        }
    }

    pub fn caret_x(&self, byte: usize) -> f32 {
        for (i, glyph) in self.glyphs.iter().enumerate() {
            if byte <= glyph.range.start {
                return glyph.pen.x;
            }

            if byte < glyph.range.end {
                let span = (glyph.range.end - glyph.range.start) as f32;
                let into = (byte - glyph.range.start) as f32;

                return glyph.pen.x + (self.advance(i) - glyph.pen.x) * (into / span);
            }
        }

        self.glyphs
            .len()
            .checked_sub(1)
            .map(|last| self.advance(last))
            .unwrap_or(0.0)
    }

    pub fn byte_at_x(&self, x: f32) -> usize {
        let mut best = 0;
        let mut best_dist = f32::INFINITY;

        for (i, glyph) in self.glyphs.iter().enumerate() {
            for (edge, byte) in [
                (glyph.pen.x, glyph.range.start),
                (self.advance(i), glyph.range.end),
            ] {
                let dist = (edge - x).abs();

                if dist < best_dist {
                    best_dist = dist;
                    best = byte;
                }
            }
        }

        best
    }
}

pub fn attrs<'a>(
    span: &TextSpan,
    family: Option<&'a str>,
    default: &ct::Attrs<'a>,
) -> ct::Attrs<'a> {
    let mut attrs = default.clone();

    if let Some(family) = family {
        attrs = attrs.family(ct::Family::Name(family));
    }

    if let Some(color) = span.color {
        attrs = attrs.color(ct::Color::rgba(
            color.r_u8(),
            color.g_u8(),
            color.b_u8(),
            color.a_u8(),
        ));
    }

    if span.bold {
        attrs = attrs.weight(ct::Weight::BOLD);
    }

    if span.italic {
        attrs = attrs.style(ct::Style::Italic);
    }

    attrs.metadata(span.metadata)
}

pub fn layout_key(spans: &[TextSpan], style: &TextStyle, size: f32) -> u64 {
    let mut h = hash_style(style, size);

    spans.len().hash(&mut h);

    for span in spans {
        hash_span(&mut h, span);
    }

    h.finish()
}

pub fn layout_key_str(text: &str, style: &TextStyle, size: f32) -> u64 {
    let mut h = hash_style(style, size);

    1usize.hash(&mut h);
    text.hash(&mut h);
    hash_span_attrs(&mut h, &TextSpan::default());

    h.finish()
}

fn hash_style(style: &TextStyle, size: f32) -> FxHasher {
    let mut h = FxHasher::default();

    style.font.hash(&mut h);
    utils::hash_f32(size, &mut h);
    utils::hash_f32(style.line_height, &mut h);
    mem::discriminant(&style.align).hash(&mut h);
    style.bold.hash(&mut h);
    style.italic.hash(&mut h);

    match style.wrap {
        Some(w) => {
            1u8.hash(&mut h);
            utils::hash_f32(w, &mut h);
        }
        None => 0u8.hash(&mut h),
    };

    h
}

fn hash_span(h: &mut FxHasher, span: &TextSpan) {
    span.text.hash(h);
    hash_span_attrs(h, span);
}

fn hash_span_attrs(h: &mut FxHasher, span: &TextSpan) {
    span.font.hash(h);
    span.bold.hash(h);
    span.italic.hash(h);
    span.metadata.hash(h);

    match span.color {
        Some(c) => {
            1u8.hash(h);
            for v in c.array() {
                utils::hash_f32(v, h);
            }
        }
        None => 0u8.hash(h),
    };
}
