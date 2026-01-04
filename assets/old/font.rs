use macros::Get;
use std::ops::Deref;
use wgpu::naga::FastHashMap;

#[derive(Debug, Clone)]
pub struct Glyph {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
#[derive(Get)]
pub struct Font {
    #[get]
    inner: fontdue::Font,

    #[get(copied)]
    size: u8,
    glyphs: FastHashMap<char, Glyph>,
}

impl Deref for Font {
    type Target = fontdue::Font;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Font {
    pub fn new(bytes: Vec<u8>, size: u8) -> Self {
        let inner = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
            .expect("Failed to load font");

        Self {
            inner,
            size,
            glyphs: FastHashMap::default(),
        }
    }

    #[inline]
    pub fn get_glyph(&self, ch: &char) -> &Glyph {
        self.glyphs.get(ch).as_ref().expect("Failed to get glyph")
    }

    #[inline]
    pub fn add_glyph(&mut self, ch: char, width: u32, height: u32) {
        self.glyphs.insert(ch, Glyph { width, height });
    }
}
