use std::rc::Rc;

use super::{font::Font, renderer::texture_creator};
use hashbrown::HashMap;
use sdl2::{
    pixels::{Color, PixelFormatEnum},
    render::{BlendMode, Texture},
    surface::Surface,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TextureKind {
    Image(Rc<str>),
    // (radius, start angle, end angle)
    Arc(u32, i32, i32),
    // (radius, start angle, end angle)
    AAArc(u32, i32, i32),
    // (radius, start angle, end angle)
    FilledArc(u32, i32, i32),
    // (radius, start angle, end angle)
    AAFilledArc(u32, i32, i32),
    // (radius)
    Circle(u32),
    // (radius)
    AACircle(u32),
    // (radius)
    FilledCircle(u32),
    // (radius)
    AAFilledCircle(u32),
}

/// Atlas is a struct that holds all the textures and fonts used in the game.
/// It is used to store the textures and fonts as 'static, so they can be used
/// in the renderer, avoiding to load them every frame.
///
/// All the textures are stored as white, and colored in the renderer, with the
/// `set_color_mod` method, so that we dont have to store the same texture with
/// different colors.
pub(crate) struct Atlas {
    pub(crate) fonts: HashMap<String, Font>,
    pub(crate) current_font: String,

    pub(crate) images: HashMap<TextureKind, Texture<'static>>,
    pub(crate) arcs: HashMap<TextureKind, Texture<'static>>,
    pub(crate) circles: HashMap<TextureKind, Texture<'static>>,
    pub(crate) aa_circles: HashMap<TextureKind, Texture<'static>>,
    pub(crate) filled_circles: HashMap<TextureKind, Texture<'static>>,
    pub(crate) aa_filled_circles: HashMap<TextureKind, Texture<'static>>,
}

impl Atlas {
    pub(crate) fn new() -> Self {
        let font = fontdue::Font::from_bytes(
            include_bytes!("../../assets/default.ttf") as &[u8],
            Default::default(),
        )
        .unwrap();

        let mut fonts = HashMap::new();

        fonts.insert("default".to_string(), Font::new(font, 18.0));

        Self {
            fonts,
            current_font: "default".to_string(),
            images: HashMap::new(),
            arcs: HashMap::new(),
            circles: HashMap::new(),
            aa_circles: HashMap::new(),
            filled_circles: HashMap::new(),
            aa_filled_circles: HashMap::new(),
        }
    }

    pub(crate) fn insert_glyph(&mut self, glyph: char) {
        let font = self.fonts.get_mut(&self.current_font).unwrap();

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
        font.char_cache.insert(glyph, texture);
    }

    pub(crate) fn get_glyph(&mut self, glyph: char) -> Option<&mut Texture<'static>> {
        let font = self.fonts.get_mut(&self.current_font).unwrap();
        font.char_cache.get_mut(&glyph)
    }

    pub(crate) fn insert_texture(&mut self, kind: &TextureKind, texture: Texture<'static>) {
        match kind {
            TextureKind::Image(label) => {
                self.images
                    .insert(TextureKind::Image(label.clone()), texture);
            }
            TextureKind::Arc(r, start, end) => {
                self.arcs
                    .insert(TextureKind::Arc(*r, *start, *end), texture);
            }
            TextureKind::AAArc(r, start, end) => {
                self.arcs
                    .insert(TextureKind::AAArc(*r, *start, *end), texture);
            }
            TextureKind::FilledArc(r, start, end) => {
                self.arcs
                    .insert(TextureKind::FilledArc(*r, *start, *end), texture);
            }
            TextureKind::AAFilledArc(r, start, end) => {
                self.arcs
                    .insert(TextureKind::AAFilledArc(*r, *start, *end), texture);
            }
            TextureKind::Circle(r) => {
                self.circles.insert(TextureKind::Circle(*r), texture);
            }
            TextureKind::AACircle(r) => {
                self.aa_circles.insert(TextureKind::AACircle(*r), texture);
            }
            TextureKind::FilledCircle(r) => {
                self.filled_circles
                    .insert(TextureKind::FilledCircle(*r), texture);
            }
            TextureKind::AAFilledCircle(r) => {
                self.aa_filled_circles
                    .insert(TextureKind::AAFilledCircle(*r), texture);
            }
        }
    }

    pub(crate) fn get_texture(&mut self, kind: &TextureKind) -> Option<&mut Texture<'static>> {
        match kind {
            TextureKind::Image(path) => self.images.get_mut(&TextureKind::Image(path.clone())),
            TextureKind::Arc(r, start, end) => {
                self.arcs.get_mut(&TextureKind::Arc(*r, *start, *end))
            }
            TextureKind::AAArc(r, start, end) => {
                self.arcs.get_mut(&TextureKind::AAArc(*r, *start, *end))
            }
            TextureKind::FilledArc(r, start, end) => {
                self.arcs.get_mut(&TextureKind::FilledArc(*r, *start, *end))
            }
            TextureKind::AAFilledArc(r, start, end) => self
                .arcs
                .get_mut(&TextureKind::AAFilledArc(*r, *start, *end)),
            TextureKind::Circle(r) => self.circles.get_mut(&TextureKind::Circle(*r)),
            TextureKind::AACircle(r) => self.aa_circles.get_mut(&TextureKind::AACircle(*r)),
            TextureKind::FilledCircle(r) => {
                self.filled_circles.get_mut(&TextureKind::FilledCircle(*r))
            }
            TextureKind::AAFilledCircle(r) => self
                .aa_filled_circles
                .get_mut(&TextureKind::AAFilledCircle(*r)),
        }
    }
}
