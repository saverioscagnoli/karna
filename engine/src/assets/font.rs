use std::path::Path;
use std::time::Duration;
use std::time::Instant;

use gpu::Filter;
use logging::warn;
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rasterize {
    Crisp,
    #[default]
    Antialiased,
}

fn threshold(pixels: &mut [u8]) {
    for px in pixels.chunks_exact_mut(4) {
        px[3] = if px[3] >= 128 { 255 } else { 0 };
    }
}

fn rasterize(font: &mut ttf::Font, mode: Rasterize) -> (Vec<GlyphBitmap>, u32) {
    let mut glyphs = Vec::new();
    let (mut inked, mut off_grid) = (0usize, 0usize);

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

        let cell: math::Size<u32> = surface.size().into();

        let Ok(mut pixels) = surface_to_rgba(surface) else {
            continue;
        };

        for px in pixels.chunks_exact(4) {
            match px[3] {
                0 => {}
                255 => inked += 1,
                _ => {
                    inked += 1;
                    off_grid += 1;
                }
            }
        }

        if mode == Rasterize::Crisp {
            threshold(&mut pixels);
        }

        let advance = metrics.advance as f32;

        let Some((pixels, bearing, size)) = trim(&pixels, cell) else {
            glyphs.push(GlyphBitmap {
                ch,
                advance,
                bearing: math::Vector2::zero(),
                size: math::Size::zero(),
                pixels: Vec::new(),
            });

            continue;
        };

        glyphs.push(GlyphBitmap {
            ch,
            advance,
            bearing,
            size,
            pixels,
        });
    }

    let off_grid = (100 * off_grid / inked.max(1)) as u32;

    (glyphs, off_grid)
}

fn warn_off_grid(name: &str, point_size: f32, mode: Rasterize, off_grid: u32) {
    if mode != Rasterize::Crisp || off_grid == 0 {
        return;
    }

    warn!(
        "Font '{}' does not sit on the pixel grid at {}pt ({}% of its glyph pixels are partly \
         covered), so Rasterize::Crisp will round them unevenly. Use the size the font was drawn \
         for, or Rasterize::Antialiased.",
        name, point_size, off_grid
    );
}

fn trim(
    pixels: &[u8],
    cell: math::Size<u32>,
) -> Option<(Vec<u8>, math::Vector2<f32>, math::Size<u32>)> {
    let (w, h) = (cell.width as usize, cell.height as usize);
    let inked = |x: usize, y: usize| pixels[(y * w + x) * 4 + 3] > 0;

    let min_x = (0..w).find(|&x| (0..h).any(|y| inked(x, y)))?;
    let max_x = (0..w).rev().find(|&x| (0..h).any(|y| inked(x, y)))?;
    let min_y = (0..h).find(|&y| (0..w).any(|x| inked(x, y)))?;
    let max_y = (0..h).rev().find(|&y| (0..w).any(|x| inked(x, y)))?;

    let size = math::Size::new((max_x - min_x + 1) as u32, (max_y - min_y + 1) as u32);
    let row_bytes = size.width as usize * 4;
    let mut out = Vec::with_capacity(row_bytes * size.height as usize);

    for y in min_y..=max_y {
        let start = (y * w + min_x) * 4;
        out.extend_from_slice(&pixels[start..start + row_bytes]);
    }

    let bearing = math::Vector2::new(min_x as f32, min_y as f32);

    Some((out, bearing, size))
}

pub fn decode_font(path: &Path, point_size: f32, mode: Rasterize) -> Result<DecodedFont, String> {
    let start = Instant::now();
    let ttf = ttf::init().map_err(|e| e.to_string())?;
    let mut font = ttf.load_font(path, point_size).map_err(|e| e.to_string())?;
    let (glyphs, off_grid) = rasterize(&mut font, mode);
    let name = font
        .face_family_name()
        .unwrap_or("Unnamed font".to_string());

    warn_off_grid(&name, point_size, mode, off_grid);

    Ok(DecodedFont {
        name,
        glyphs,
        line_height: font.recommended_line_spacing() as f32,
        height: font.height() as f32,
        duration: start.elapsed(),
    })
}

pub fn decode_font_bytes(
    bytes: &[u8],
    point_size: f32,
    mode: Rasterize,
) -> Result<DecodedFont, String> {
    let start = Instant::now();
    let ttf = ttf::init().map_err(|e| e.to_string())?;
    let io = IOStream::from_bytes(bytes).map_err(|e| e.to_string())?;
    let mut font = ttf
        .load_font_from_iostream(io, point_size)
        .map_err(|e| e.to_string())?;
    let (glyphs, off_grid) = rasterize(&mut font, mode);
    let name = font
        .face_family_name()
        .unwrap_or("Unnamed font".to_string());

    warn_off_grid(&name, point_size, mode, off_grid);

    Ok(DecodedFont {
        name,
        glyphs,
        line_height: font.recommended_line_spacing() as f32,
        height: font.height() as f32,
        duration: start.elapsed(),
    })
}

pub struct GlyphBitmap {
    ch: char,
    advance: f32,
    bearing: math::Vector2<f32>,
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
    pub image: Option<Handle<Image>>,
    pub advance: f32,
    pub bearing: math::Vector2<f32>,
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
            let image = (!glyph.pixels.is_empty()).then(|| {
                let handle: Handle<Image> = images.insert(AssetSlot::Pending).cast();

                let image = atlas.insert_rgba(&glyph.pixels, glyph.size, handle, Filter::Nearest);
                images[handle.cast()] = AssetSlot::Ready(image);

                handle
            });

            glyphs.insert(
                glyph.ch,
                Glyph {
                    image,
                    advance: glyph.advance,
                    bearing: glyph.bearing,
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
