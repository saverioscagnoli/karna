use logging::debug;
use sdl3::iostream::IOStream;
use sdl3::pixels::Color;
use sdl3::surface::Surface;
use utils::FastHashMap;
use utils::Handle;

use crate::assets::atlas::Image;
use crate::assets::atlas::TextureAtlas;
use crate::assets::surface_to_rgba;

/// Glyph used when a character is missing from the baked set.
const FALLBACK: char = '?';

#[derive(Debug, Clone)]
pub struct Glyph {
    /// Atlas entry for the glyph cell. `Handle::INVALID` for glyphs with no
    /// visible pixels (e.g. space), which still advance the pen.
    pub image: Handle<Image>,
    pub advance: f32,
}

#[derive(Debug, Clone)]
pub struct Font {
    glyphs: FastHashMap<char, Glyph>,
    line_height: f32,
    height: f32,
}

impl Font {
    pub(crate) fn bake(atlas: &mut TextureAtlas, data: &[u8], pt_size: f32) -> Self {
        let start = std::time::Instant::now();

        // Ttf context::new just does a fetch_add
        let ttf = sdl3::ttf::init().expect("Failed to initialize SDL3_ttf");
        let io = IOStream::from_bytes(data).expect("Failed to create a stream from font bytes");
        let font = ttf
            .load_font_from_iostream(io, pt_size)
            .expect("Failed to load font");

        let mut glyphs = FastHashMap::default();

        for ch in '\0'..=char::MAX {
            if ch.is_control() || font.find_glyph(ch).is_none() {
                continue;
            }

            let Some(metrics) = font.find_glyph_metrics(ch) else {
                continue;
            };

            let image = match font.render_char(ch).blended(Color::WHITE) {
                Ok(surface) => Self::insert_cell(atlas, surface),
                Err(_) => Handle::INVALID,
            };

            glyphs.insert(
                ch,
                Glyph {
                    image,
                    advance: metrics.advance as f32,
                },
            );
        }

        debug!(
            "Baked {} glyphs ({:?}, {}pt) in {:?}",
            glyphs.len(),
            font.face_family_name(),
            pt_size,
            start.elapsed()
        );

        Self {
            glyphs,
            line_height: font.recommended_line_spacing() as f32,
            height: font.height() as f32,
        }
    }

    fn insert_cell(atlas: &mut TextureAtlas, surface: Surface) -> Handle<Image> {
        let size = math::Size::new(surface.width(), surface.height());
        let data = surface_to_rgba(surface);

        // Fully transparent cells (space & friends) don't earn atlas space.
        if data.chunks_exact(4).all(|px| px[3] == 0) {
            return Handle::INVALID;
        }

        atlas.insert(&data, size)
    }

    pub fn glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyphs.get(&ch).or_else(|| self.glyphs.get(&FALLBACK))
    }

    /// Vertical distance between the tops of two consecutive lines.
    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    /// Height of a single glyph cell.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Size of the bounding box `text` would occupy when drawn.
    pub fn measure(&self, text: &str) -> math::Size<f32> {
        let mut width = 0.0f32;
        let mut line_width = 0.0;
        let mut lines = 1;

        for ch in text.chars() {
            if ch == '\n' {
                width = width.max(line_width);
                line_width = 0.0;
                lines += 1;
                continue;
            }

            if let Some(glyph) = self.glyph(ch) {
                line_width += glyph.advance;
            }
        }

        width = width.max(line_width);

        math::Size::new(width, (lines - 1) as f32 * self.line_height + self.height)
    }
}
