use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use sdl3::iostream::IOStream;
use sdl3::pixels::Color;
use sdl3::ttf;
use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

use crate::Image;
use crate::assets::AssetSlot;
use crate::assets::atlas::TextureAtlas;
use crate::assets::surface_to_rgba;

const FALLBACK: char = '?';

pub fn decode_font(path: &Path, point_size: f32) -> Result<DecodedFont, String> {
    let start = Instant::now();
    let ttf = ttf::init().map_err(|e| e.to_string())?;
    let font = ttf.load_font(path, point_size).map_err(|e| e.to_string())?;
    let mut glyphs = Vec::new();

    for ch in '\0'..char::MAX {
        if ch.is_control() || font.find_glyph(ch).is_none() {
            continue;
        }

        let Some(metrics) = font.find_glyph_metrics(ch) else {
            continue;
        };

        let Ok(surface) = font.render_char(ch).blended(Color::WHITE) else {
            continue;
        };

        let size: math::Size<u32> = surface.size().into();

        let Ok(data) = surface_to_rgba(surface) else {
            continue;
        };

        glyphs.push(GlyphBitmap {
            ch,
            advance: metrics.advance as f32,
            size,
            pixels: data,
        });
    }

    Ok(DecodedFont {
        name: font
            .face_family_name()
            .unwrap_or("Unnamed font".to_string()),
        glyphs,
        line_height: font.recommended_line_spacing() as f32,
        height: font.height() as f32,
        duration: start.elapsed(),
    })
}

pub fn decode_font_bytes(bytes: &[u8], point_size: f32) -> Result<DecodedFont, String> {
    let start = Instant::now();
    let ttf = ttf::init().map_err(|e| e.to_string())?;
    let io = IOStream::from_bytes(bytes).map_err(|e| e.to_string())?;
    let font = ttf
        .load_font_from_iostream(io, point_size)
        .map_err(|e| e.to_string())?;
    let mut glyphs = Vec::new();

    for ch in '\0'..char::MAX {
        if ch.is_control() || font.find_glyph(ch).is_none() {
            continue;
        }

        let Some(metrics) = font.find_glyph_metrics(ch) else {
            continue;
        };

        let Ok(surface) = font.render_char(ch).blended(Color::WHITE) else {
            continue;
        };

        let size: math::Size<u32> = surface.size().into();

        let Ok(data) = surface_to_rgba(surface) else {
            continue;
        };

        glyphs.push(GlyphBitmap {
            ch,
            advance: metrics.advance as f32,
            size,
            pixels: data,
        });
    }

    Ok(DecodedFont {
        name: font
            .face_family_name()
            .unwrap_or("Unnamed font".to_string()),
        glyphs,
        line_height: font.recommended_line_spacing() as f32,
        height: font.height() as f32,
        duration: start.elapsed(),
    })
}

pub struct GlyphBitmap {
    ch: char,
    advance: f32,
    size: math::Size<u32>,
    pixels: Vec<u8>,
}

pub struct DecodedFont {
    pub name: String,
    pub glyphs: Vec<GlyphBitmap>,
    pub line_height: f32,
    pub height: f32,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub image: Handle<Image>,
    pub advance: f32,
}

pub struct Font {
    glyphs: FastHashMap<char, Glyph>,
    line_height: f32,
    height: f32,
}

impl Font {
    pub(crate) fn bake(
        atlas: &mut TextureAtlas,
        images: &mut SlotMap<AssetSlot<Image>>,
        decoded: DecodedFont,
    ) -> Self {
        let mut glyphs = FastHashMap::default();

        for glyph in decoded.glyphs {
            let handle: Handle<Image> = images.insert(AssetSlot::Pending).cast();
            let image = atlas.insert_rgba(&glyph.pixels, glyph.size, handle);

            images[handle.cast()] = AssetSlot::Ready(image);

            glyphs.insert(
                glyph.ch,
                Glyph {
                    image: handle,
                    advance: glyph.advance,
                },
            );
        }

        Self {
            glyphs,
            line_height: decoded.line_height,
            height: decoded.height,
        }
    }

    pub fn glyph(&self, ch: char) -> Option<&Glyph> {
        self.glyphs.get(&ch).or_else(|| self.glyphs.get(&FALLBACK))
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}
