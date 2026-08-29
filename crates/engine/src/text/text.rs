use cosmic_text::Attrs;
use cosmic_text::Family;
use cosmic_text::Style;
use cosmic_text::Weight;
use math as m;

use utils::Handle;

use crate::Color;
use crate::text::font::Font;
use crate::text::glyph::PositionedGlyph;
pub use cosmic_text::Align as TextAlign;

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font: Option<Handle<Font>>,
    pub size: f32,
    pub line_height: f32,
    pub wrap: Option<f32>,
    pub align: TextAlign,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: None,
            size: 16.0,
            line_height: 20.0,
            wrap: None,
            align: TextAlign::Left,
        }
    }
}

impl TextStyle {
    pub fn new(font: Handle<Font>, size: f32) -> Self {
        Self {
            font: Some(font),
            size,
            line_height: size * 1.25,
            ..Self::default()
        }
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }

    pub fn with_wrap(mut self, width: f32) -> Self {
        self.wrap = Some(width);
        self
    }

    pub fn with_align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextSpan<'a> {
    pub text: &'a str,
    pub color: Option<Color>,
    pub font: Option<Handle<Font>>,
    pub bold: bool,
    pub italic: bool,
    pub metadata: usize,
}

impl<'a> TextSpan<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
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

pub struct Text {
    pub(crate) content: String,
    pub(crate) glyphs: Vec<PositionedGlyph>,
    pub(crate) size: m::Size<f32>,
}

impl Text {
    pub fn as_str(&self) -> &str {
        &self.content
    }

    pub fn glyphs(&self) -> &[PositionedGlyph] {
        &self.glyphs
    }

    pub fn size(&self) -> m::Size<f32> {
        self.size
    }

    fn advance(&self, index: usize) -> f32 {
        match self.glyphs.get(index + 1) {
            Some(next) if next.line == self.glyphs[index].line => next.pen.x,
            _ => self.size.width,
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

impl Into<Color> for cosmic_text::Color {
    fn into(self) -> Color {
        Color {
            r: self.r() as f32 / 255.0,
            g: self.g() as f32 / 255.0,
            b: self.b() as f32 / 255.0,
            a: self.a() as f32 / 255.0,
        }
    }
}

impl Into<cosmic_text::Color> for Color {
    fn into(self) -> cosmic_text::Color {
        let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;

        cosmic_text::Color::rgba(b(self.r), b(self.g), b(self.b), b(self.a))
    }
}

pub fn attrs<'a>(
    span: &TextSpan<'a>,
    family: Option<&'a str>,
    default: &Attrs<'a>,
) -> Attrs<'a> {
    let mut attrs = default.clone();

    if let Some(family) = family {
        attrs = attrs.family(Family::Name(family));
    }

    if let Some(color) = span.color {
        attrs = attrs.color(color.into());
    }

    if span.bold {
        attrs = attrs.weight(Weight::BOLD);
    }

    if span.italic {
        attrs = attrs.style(Style::Italic);
    }

    attrs.metadata(span.metadata)
}
