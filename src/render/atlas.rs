use super::renderer::texture_creator;
use fontdue::layout::{CoordinateSystem, Layout};
use sdl2::{
    pixels::{Color, PixelFormatEnum},
    render::{BlendMode, Texture},
    surface::Surface,
};
use std::{collections::HashMap, ops::Deref, rc::Rc};

pub(crate) struct Font {
    pub(crate) inner: Rc<fontdue::Font>,
    pub(crate) size: f32,
}

impl Deref for Font {
    type Target = fontdue::Font;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub(crate) struct Atlas {
    pub(crate) fonts: HashMap<String, (Font, HashMap<char, Texture<'static>>)>,
    pub(crate) current_font: String,
    pub(crate) layout: Layout,
}

impl Atlas {
    pub fn new() -> Self {
        let font = fontdue::Font::from_bytes(
            include_bytes!("../../examples/assets/font.ttf") as &[u8],
            Default::default(),
        )
        .unwrap();

        let mut fonts = HashMap::new();

        fonts.insert(
            "default".to_string(),
            (
                Font {
                    inner: Rc::new(font),
                    size: 18.0,
                },
                HashMap::new(),
            ),
        );

        Atlas {
            fonts,
            current_font: "default".to_string(),
            layout: Layout::new(CoordinateSystem::PositiveYDown),
        }
    }

    pub fn insert_glyph(&mut self, glyph: char) {
        let (font, char_cache) = self.fonts.get_mut(&self.current_font).unwrap();

        let (metrics, bitmap) = font.rasterize(glyph, font.size as f32);
        let (width, height) = (metrics.width as u32, metrics.height as u32);

        if width == 0 || height == 0 {
            return;
        }

        let mut surface = Surface::new(width, height, PixelFormatEnum::RGBA32).unwrap();

        for y in 0..height {
            for x in 0..width {
                let i = (y * width + x) as usize;
                let alpha = bitmap[i];
                let pixel = Color::RGBA(255, 255, 255, alpha);
                surface.with_lock_mut(|pixels| {
                    let offset = (y * width + x) as usize * 4;
                    pixels[offset] = pixel.r;
                    pixels[offset + 1] = pixel.g;
                    pixels[offset + 2] = pixel.b;
                    pixels[offset + 3] = pixel.a;
                });
            }
        }

        let mut texture = texture_creator()
            .create_texture_from_surface(surface)
            .unwrap();

        texture.set_blend_mode(BlendMode::Blend);
        char_cache.insert(glyph, texture);
    }

    pub(crate) fn get_glyph(&mut self, glyph: char) -> Option<&mut Texture<'static>> {
        let (_, char_cache) = self.fonts.get_mut(&self.current_font).unwrap();
        char_cache.get_mut(&glyph)
    }
}
