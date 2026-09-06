mod font;
mod glyph;
mod text;

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use cosmic_text::Attrs;
use cosmic_text::Buffer;
use cosmic_text::CacheKey;
use cosmic_text::Family;
use cosmic_text::FontSystem;
use cosmic_text::Metrics;
use cosmic_text::Shaping;
use cosmic_text::SwashCache;
use cosmic_text::SwashContent;
use cosmic_text::Wrap;
use cosmic_text::fontdb as fdb;
use logging::fatal;
use logging::warn;
use math as m;
use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

use crate::assets::TextureAtlas;
use crate::gpu::Filter;
use crate::text::glyph::CachedGlyph;
use crate::text::glyph::PositionedGlyph;
use crate::text::text::attrs;

pub use crate::text::font::Font;
pub use crate::text::text::Text;
pub use crate::text::text::TextAlign;
pub use crate::text::text::TextSpan;
pub use crate::text::text::TextStyle;
use crate::text::text::layout_key;

struct CachedLayout {
    text: Arc<Text>,
    used: u64,
}

pub struct TextSystem {
    fonts: FontSystem,
    scratch: Buffer,
    swash: SwashCache,
    glyphs: FastHashMap<CacheKey, Option<CachedGlyph>>,
    registry: SlotMap<Font>,
    paths: FastHashMap<PathBuf, Handle<Font>>,
    families: FastHashMap<String, Handle<Font>>,
    layouts: FastHashMap<u64, CachedLayout>,
    frame: u64,
}

impl Default for TextSystem {
    fn default() -> Self {
        let mut fonts = FontSystem::new();
        let scratch = Buffer::new(&mut fonts, Metrics::new(16.0, 20.0));
        Self {
            fonts,
            scratch,
            swash: SwashCache::new(),
            glyphs: FastHashMap::default(),
            registry: SlotMap::new(),
            paths: FastHashMap::default(),
            families: FastHashMap::default(),
            layouts: FastHashMap::default(),
            frame: 0,
        }
    }
}

impl TextSystem {
    pub fn family(&self, font: Handle<Font>) -> Option<&str> {
        self.registry.get(font).map(|font| font.family.as_str())
    }

    fn intern(&mut self, family: String) -> Handle<Font> {
        if let Some(handle) = self.families.get(&family) {
            return *handle;
        }

        let handle = self.registry.insert(Font {
            family: family.clone(),
        });

        self.families.insert(family, handle);

        handle
    }

    pub fn system(&mut self, name: &str) -> Handle<Font> {
        let known = self
            .fonts
            .db()
            .faces()
            .any(|face| face.families.iter().any(|(family, _)| family == name));

        if !known {
            warn!(
                "No system font family named '{}', text will fall back.",
                name
            );
        }

        self.intern(name.to_owned())
    }

    pub fn register_bytes(&mut self, bytes: &[u8]) -> Handle<Font> {
        let source = fdb::Source::Binary(Arc::new(bytes.to_vec()));
        let ids = self.fonts.db_mut().load_font_source(source);

        let Some(id) = ids.first() else {
            fatal!("No font faces found in font data");
        };

        let family = self
            .fonts
            .db()
            .face(*id)
            .and_then(|face| face.families.first().map(|(name, _)| name.clone()));

        let Some(family) = family else {
            fatal!("Font face has no family name");
        };

        self.intern(family)
    }

    pub fn register_path(&mut self, path: &Path) -> Handle<Font> {
        if let Some(handle) = self.paths.get(path) {
            return *handle;
        }

        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => fatal!("Failed to read font '{}': {}", path.display(), e),
        };

        let handle = self.register_bytes(&bytes);

        self.paths.insert(path.to_path_buf(), handle);

        handle
    }

    fn glyph_rgba(data: &[u8], content: SwashContent, size: m::Size<u32>) -> Vec<u8> {
        let pixels = size.area() as usize;

        match content {
            SwashContent::Mask => {
                let mut out = Vec::with_capacity(pixels * 4);

                for &coverage in data.iter().take(pixels) {
                    out.extend_from_slice(&[255, 255, 255, coverage]);
                }

                out
            }

            SwashContent::Color | SwashContent::SubpixelMask => data[..pixels * 4].to_vec(),
        }
    }

    fn glyph(&mut self, key: CacheKey, atlas: &mut TextureAtlas) -> Option<CachedGlyph> {
        if let Some(cached) = self.glyphs.get(&key) {
            return *cached;
        }

        let rasterized = match self.swash.get_image(&mut self.fonts, key) {
            Some(image) if image.placement.width > 0 && image.placement.height > 0 => {
                let size = m::Size::new(image.placement.width, image.placement.height);

                Some((
                    Self::glyph_rgba(&image.data, image.content, size),
                    size,
                    m::Vector2::new(image.placement.left as f32, -image.placement.top as f32),
                    matches!(image.content, SwashContent::Mask),
                ))
            }

            _ => None,
        };

        let cached = rasterized.map(|(pixels, size, placement, mask)| CachedGlyph {
            image: atlas.insert_rgba(&pixels, size, Handle::INVALID, Filter::Nearest),
            placement,
            size,
            colored: !mask,
        });

        self.glyphs.insert(key, cached);
        cached
    }

    pub fn layout_rich(
        &mut self,
        spans: &[TextSpan],
        style: &TextStyle,
        atlas: &mut TextureAtlas,
    ) -> Arc<Text> {
        let key = layout_key(spans, style);

        if let Some(entry) = self.layouts.get_mut(&key) {
            entry.used = self.frame;
            return entry.text.clone();
        }

        let text = Arc::new(self.shape(spans, style, atlas));

        self.layouts.insert(
            key,
            CachedLayout {
                text: text.clone(),
                used: self.frame,
            },
        );

        text
    }

    fn shape(&mut self, spans: &[TextSpan], style: &TextStyle, atlas: &mut TextureAtlas) -> Text {
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

        let default = match &default_family {
            Some(family) => Attrs::new().family(Family::Name(family)),
            None => Attrs::new(),
        };

        let content = spans.iter().map(|span| span.text).collect::<String>();
        let mut buffer = self.scratch.borrow_with(&mut self.fonts);

        buffer.set_metrics(Metrics::new(style.size, style.line_height));

        buffer.set_wrap(if style.wrap.is_some() {
            Wrap::WordOrGlyph
        } else {
            Wrap::None
        });

        buffer.set_size(style.wrap, None);
        buffer.set_rich_text(
            spans
                .iter()
                .zip(&span_families)
                .map(|(span, family)| (span.text, attrs(span, family.as_deref(), &default))),
            &default,
            Shaping::Advanced,
            Some(style.align),
        );

        let mut glyphs = Vec::new();
        let mut keys = Vec::new();
        let mut size = m::Size::new(0.0, 0.0).cast::<f32>();

        for (line, run) in buffer.layout_runs().enumerate() {
            size.width = size.width.max(run.line_w);
            size.height = size.height.max(run.line_top + run.line_height);

            for glyph in run.glyphs {
                let physical = glyph.physical((0.0, 0.0), 1.0);

                keys.push(physical.cache_key);

                let pen = m::Vector2::new(physical.x as f32, run.line_y + physical.y as f32);

                glyphs.push(PositionedGlyph {
                    image: None,
                    pos: pen,
                    pen,
                    size: m::Size::zero(),
                    range: glyph.start..glyph.end,
                    line,
                    color: glyph.color_opt.map(cosmic_text::Color::into),
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

        Text {
            content,
            glyphs,
            size,
        }
    }

    pub fn layout<T>(&mut self, text: T, style: &TextStyle, atlas: &mut TextureAtlas) -> Arc<Text>
    where
        T: AsRef<str>,
    {
        self.layout_rich(&[TextSpan::new(text.as_ref())], style, atlas)
    }

    pub(crate) fn begin_frame(&mut self) {
        self.frame += 1;

        if self.frame % 60 == 0 {
            let cutoff = self.frame.saturating_sub(120);
            self.layouts.retain(|_, entry| entry.used >= cutoff);
        }
    }
}
