use ab_glyph::Font as AbGlyphFont;
use ab_glyph::ScaleFont;
use logging::debug;
use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

use crate::image::TextureAtlas;

pub struct Font {
    pub inner: ab_glyph::FontVec,
    size: u16,
}

impl Font {
    #[doc(hidden)]
    pub fn _inner(&self) -> &ab_glyph::FontVec {
        &self.inner
    }

    fn chars(&self) -> Vec<char> {
        self.inner.codepoint_ids().map(|(_, ch)| ch).collect()
    }

    pub fn size(&self) -> u16 {
        self.size
    }
}

#[derive(Hash, Eq, PartialEq)]
pub struct GlyphKey {
    pub font: Handle<Font>,
    pub size_px: u16,
    pub ch: char,
}

#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    pub uv: math::Vector4<f32>,
    pub size: math::Size<f32>,
    pub bearing: math::Vector2<f32>,
    pub advance: f32,
}

pub struct FontAtlas {
    pub(super) fonts: SlotMap<Font>,
    pub(super) glyphs: FastHashMap<GlyphKey, GlyphInfo>,
    pub(super) debug_font: Handle<Font>,
}

impl FontAtlas {
    pub fn new(tex_atlas: &mut TextureAtlas) -> Self {
        let mut font_atlas = Self {
            fonts: SlotMap::new(),
            glyphs: FastHashMap::default(),
            debug_font: Handle::default(),
        };

        let bytes = include_bytes!("../../bundled/DOS-V.ttf");
        let debug_font =
            ab_glyph::FontVec::try_from_vec(bytes.to_vec()).expect("Failed to load debug font");

        let debug_font = font_atlas.register_font(tex_atlas, debug_font, 16);

        font_atlas.debug_font = debug_font;
        font_atlas
    }

    pub fn register_font(
        &mut self,
        atlas: &mut TextureAtlas,
        font: ab_glyph::FontVec,
        size: u16,
    ) -> Handle<Font> {
        let font_handle = self.fonts.insert(Font { inner: font, size });
        let font = self.fonts.get(font_handle).unwrap();

        for ch in font.chars() {
            self.rasterize(atlas, font_handle, size as f32, ch);
        }

        font_handle
    }

    pub fn rasterize(
        &mut self,
        atlas: &mut TextureAtlas,
        font: Handle<Font>,
        size_px: f32,
        ch: char,
    ) -> Option<GlyphInfo> {
        let key = GlyphKey {
            font,
            size_px: size_px as u16,
            ch,
        };

        let font = self.fonts.get(font).expect("Failed to get font");
        let (bitmap, size, bounds) = rasterize_glyph(font, ch, size_px)?;

        let rgba = bitmap.iter().flat_map(|&a| [255, 255, 255, a]).collect();

        let handle = atlas.load_raw(rgba, size, true);
        let uv = atlas.images.get(handle).unwrap().uv;

        let scaled = font._inner().as_scaled(ab_glyph::PxScale::from(size_px));
        let advance = scaled.h_advance(font._inner().glyph_id(ch));

        let info = GlyphInfo {
            uv,
            size: size.as_f32(),
            bearing: math::Vector2::new(bounds.min.x, bounds.min.y),
            advance,
        };

        self.glyphs.insert(key, info);

        Some(info)
    }
}

fn rasterize_glyph(
    font: &Font,
    ch: char,
    size_px: f32,
) -> Option<(Vec<u8>, math::Size<u32>, ab_glyph::Rect)> {
    let scale = ab_glyph::PxScale::from(size_px);
    let scaled_font = font._inner().as_scaled(scale);

    let glyph_id = scaled_font.glyph_id(ch);
    let glyph = glyph_id.with_scale(scale);
    let outlined = scaled_font.outline_glyph(glyph)?;
    let bounds = outlined.px_bounds();

    let width = bounds.width() as u32;
    let height = bounds.height() as u32;
    let mut bitmap = vec![0u8; (width * height) as usize];

    outlined.draw(|x, y, coverage| {
        let idx = (y * width + x) as usize;
        bitmap[idx] = (coverage * 255.0) as u8;
    });

    Some((bitmap, math::Size::new(width, height), bounds))
}
