mod font;
mod glyph;
mod text;

pub use font::*;
pub use glyph::*;
pub use text::*;

use cosmic_text as ct;
use cosmic_text::fontdb as fdb;
use math::SdlFloat;
use nostd::alloc::borrow::ToOwned;
use nostd::alloc::string::String;
use nostd::alloc::sync::Arc;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::HashMap;
use nostd::collections::SlotMap;
use nostd::fs;
use nostd::path::Path;
use nostd::path::PathBuf;
use sdl3::render::Color;
use traccia::error;

use crate::assets::TextureAtlas;

#[derive(Clone)]
struct CachedLayout {
    text: Arc<TextLayout>,
    used: u64,
}

pub struct TextSystem {
    fonts: ct::FontSystem,
    scratch: ct::Buffer,
    swash: ct::SwashCache,
    glyphs: HashMap<ct::CacheKey, Option<CachedGlyph>>,
    registry: SlotMap<Font>,
    paths: HashMap<PathBuf, Handle<Font>>,
    families: HashMap<String, Handle<Font>>,
    layouts: HashMap<u64, CachedLayout>,
    frame: u64,
    default_font: Handle<Font>,
    pub(crate) debug_font: Handle<Font>,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self {
            fonts: ct::FontSystem::new(),
            scratch: ct::Buffer::new_empty(ct::Metrics::new(16.0, 20.0)),
            swash: ct::SwashCache::new(),
            glyphs: HashMap::default(),
            registry: SlotMap::default(),
            paths: HashMap::default(),
            families: HashMap::default(),
            layouts: HashMap::default(),
            frame: 0,
            default_font: Handle::INVALID,
            debug_font: Handle::INVALID,
        }
    }
}

impl TextSystem {
    pub const DEBUG_FONT_BYTES: &[u8] = include_bytes!("../../../../assets/DOS-V.ttf");
    pub const DEBUG_FONT_SIZE: f32 = 16.0;
    pub const DEFAULT_FONT_SIZE: f32 = 16.0;

    pub fn set_default_font(&mut self, font: Handle<Font>) {
        if let Some(family) = self.registry.get(font).map(|f| f.family.clone()) {
            self.fonts.db_mut().set_sans_serif_family(family);
            self.default_font = font;
            self.layouts.clear();
        }
    }

    #[inline]
    pub fn family(&self, font: Handle<Font>) -> Option<&str> {
        self.registry.get(font).map(|f| f.family.as_str())
    }

    fn font_size(&self, style: &TextStyle) -> f32 {
        style
            .size
            .or_else(|| {
                style
                    .font
                    .and_then(|font| self.registry.get(font))
                    .or_else(|| self.registry.get(self.default_font))
                    .map(|font| font.size)
            })
            .unwrap_or(Self::DEFAULT_FONT_SIZE)
    }

    fn intern(&mut self, family: String, size: f32) -> Handle<Font> {
        if let Some(handle) = self.families.get(&family).copied() {
            if let Some(font) = self.registry.get_mut(handle) {
                font.size = size;
            }

            return handle;
        }

        let handle = self.registry.insert(Font {
            family: family.clone(),
            size,
        });

        self.families.insert(family, handle);
        handle
    }

    pub fn register_path(&mut self, path: &Path, size: f32) -> Handle<Font> {
        if let Some(handle) = self.paths.get(path).copied() {
            if let Some(font) = self.registry.get_mut(handle) {
                font.size = size;
            }

            return handle;
        };

        let blob = match fs::read(path) {
            Ok(blob) => blob,
            Err(e) => {
                error!("Failed to read font file: {}", e);
                return Handle::INVALID;
            }
        };

        let handle = self.register_bytes(&blob, size);

        self.paths.insert(path.to_path_buf(), handle);
        handle
    }

    pub fn register_bytes(&mut self, bytes: &[u8], size: f32) -> Handle<Font> {
        let source = fdb::Source::Binary(Arc::new(bytes.to_vec()));
        let ids = self.fonts.db_mut().load_font_source(source);

        let Some(id) = ids.first() else {
            error!("No font faces found in font data!");
            return Handle::INVALID;
        };

        let family = self
            .fonts
            .db()
            .face(*id)
            .and_then(|face| face.families.first().map(|(name, _)| name.clone()));

        let Some(family) = family else {
            error!("No font family found in font data!");
            return Handle::INVALID;
        };

        self.intern(family, size)
    }

    fn glyph_rgba(data: &[u8], content: ct::SwashContent, size: math::Size<u32>) -> Vec<u8> {
        let pixels = size.area() as usize;

        match content {
            ct::SwashContent::Mask => {
                let mut out = Vec::with_capacity(pixels * 4);

                for &coverage in data.iter().take(pixels) {
                    out.extend_from_slice(&[255, 255, 255, coverage]);
                }

                out
            }
            ct::SwashContent::Color | ct::SwashContent::SubpixelMask => data[..pixels * 4].to_vec(),
        }
    }

    fn glyph(&mut self, key: ct::CacheKey, atlas: &mut TextureAtlas) -> Option<CachedGlyph> {
        if let Some(cached) = self.glyphs.get(&key) {
            return *cached;
        }

        let rasterized = match self.swash.get_image(&mut self.fonts, key) {
            Some(image) if image.placement.width > 0 && image.placement.height > 0 => {
                let size = math::size!(image.placement.width, image.placement.height);

                Some((
                    Self::glyph_rgba(&image.data, image.content, size),
                    size,
                    math::vec2!(image.placement.left as f32, -image.placement.top as f32),
                    matches!(image.content, ct::SwashContent::Mask),
                ))
            }

            _ => None,
        };

        let cached = rasterized.map(|(pixels, size, placement, mask)| CachedGlyph {
            image: atlas
                .insert_rgba(&pixels, size)
                .unwrap_or_else(|| panic!("Fah")),
            placement,
            size,
            colored: !mask,
        });

        self.glyphs.insert(key, cached);
        cached
    }

    fn shape(
        &mut self,
        spans: &[TextSpan],
        style: &TextStyle,
        size: f32,
        atlas: &mut TextureAtlas,
    ) -> TextLayout {
        let default_family = style
            .font
            .and_then(|font| self.family(font))
            .map(str::to_owned);

        let span_families = spans
            .iter()
            .map(|span| {
                span.font
                    .and_then(|font| self.family(font))
                    .map(str::to_owned)
            })
            .collect::<Vec<_>>();

        let mut default = match &default_family {
            Some(family) => ct::Attrs::new().family(fdb::Family::Name(family)),
            None => ct::Attrs::new(),
        };

        if style.bold {
            default = default.weight(ct::Weight::BOLD);
        }

        if style.italic {
            default = default.style(ct::Style::Italic);
        }

        let mut buffer = self.scratch.borrow_with(&mut self.fonts);

        buffer.set_metrics(ct::Metrics::new(size, size * style.line_height));
        buffer.set_wrap(if style.wrap.is_some() {
            ct::Wrap::WordOrGlyph
        } else {
            ct::Wrap::None
        });

        buffer.set_size(style.wrap, None);
        buffer.set_rich_text(
            spans.iter().zip(&span_families).map(|(span, family)| {
                (span.text.as_str(), attrs(span, family.as_deref(), &default))
            }),
            &default,
            ct::Shaping::Advanced,
            Some(style.align),
        );

        let mut glyphs = Vec::new();
        let mut keys = Vec::new();
        let mut size = math::size!(0.0, 0.0);

        for (line, run) in buffer.layout_runs().enumerate() {
            size.width = size.w().sdl_max(run.line_w);
            size.height = size.h().sdl_max(run.line_top + run.line_height);

            for glyph in run.glyphs {
                let physical = glyph.physical((0.0, 0.0), 1.0);
                let pen = math::vec2!(physical.x as f32, run.line_y + physical.y as f32);

                keys.push(physical.cache_key);

                glyphs.push(PositionedGlyph {
                    image: None,
                    pos: pen,
                    pen,
                    size: math::Size::zero(),
                    range: glyph.start..glyph.end,
                    line,
                    color: glyph
                        .color_opt
                        .map(|c| Color::rgba_u8(c.r(), c.g(), c.b(), c.a())),
                    colored: false,
                    metadata: glyph.metadata,
                });
            }
        }

        drop(buffer);

        for (positioned, key) in glyphs.iter_mut().zip(keys) {
            let Some(cached) = self.glyph(key, atlas) else {
                continue;
            };

            positioned.image = Some(cached.image);
            positioned.pos += cached.placement;
            positioned.size = cached.size;
            positioned.colored = cached.colored;
        }

        TextLayout { glyphs, size }
    }

    pub(crate) fn layout(
        &mut self,
        spans: &[TextSpan],
        style: &TextStyle,
        atlas: &mut TextureAtlas,
    ) -> Arc<TextLayout> {
        let size = self.font_size(style);
        let key = layout_key(spans, style, size);

        self.cached(key, |this| this.shape(spans, style, size, atlas))
    }

    pub(crate) fn layout_str(
        &mut self,
        text: &str,
        style: &TextStyle,
        atlas: &mut TextureAtlas,
    ) -> Arc<TextLayout> {
        let size = self.font_size(style);
        let key = layout_key_str(text, style, size);

        self.cached(key, |this| {
            this.shape(&[TextSpan::new(text)], style, size, atlas)
        })
    }

    fn cached<F>(&mut self, key: u64, shape: F) -> Arc<TextLayout>
    where
        F: FnOnce(&mut Self) -> TextLayout,
    {
        if let Some(entry) = self.layouts.get_mut(&key) {
            entry.used = self.frame;
            return Arc::clone(&entry.text);
        }

        let text = Arc::new(shape(self));

        self.layouts.insert(
            key,
            CachedLayout {
                text: Arc::clone(&text),
                used: self.frame,
            },
        );

        text
    }

    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;

        if self.frame % 60 == 0 {
            let cutoff = self.frame.saturating_sub(120);
            self.layouts.retain(|_, entry| entry.used >= cutoff);
        }
    }
}
