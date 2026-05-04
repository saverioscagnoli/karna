use macros::Get;
use math::Size;
use utils::FastHashMap;

#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    pub width: u32,
    pub height: u32,
}

#[derive(Get)]
pub struct Font {
    #[get]
    inner: fontdue::Font,

    #[get(copied)]
    size: u16,
    glyphs: FastHashMap<char, Glyph>,
}

impl Font {
    pub fn new(bytes: &[u8], size: u16) -> Self {
        Self {
            inner: fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default())
                .expect("Cannot load font"),
            size,
            glyphs: FastHashMap::default(),
        }
    }

    pub fn chars(&self) -> Vec<char> {
        self.inner.chars().keys().copied().collect::<Vec<_>>()
    }

    pub fn rasterize_char(&mut self, ch: char) -> (Size<u32>, Vec<u8>) {
        let (metrics, bitmap) = self.inner.rasterize(ch, self.size as f32);
        let size = Size::new(metrics.width as u32, metrics.height as u32);

        self.glyphs.insert(
            ch,
            Glyph {
                width: size.width,
                height: size.height,
            },
        );

        (size, bitmap)
    }
}
