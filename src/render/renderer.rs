use super::atlas::{Atlas, Font};
use crate::math::{Size, Vec2};
use fontdue::layout::TextStyle;
use image::GenericImageView;
use sdl2::{
    pixels::{Color, PixelFormatEnum},
    rect::{FPoint, FRect},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
};
use std::{collections::HashMap, ops::Deref, path::Path, rc::Rc, sync::OnceLock};

struct TextureCreatorWrapper(TextureCreator<WindowContext>);

impl Deref for TextureCreatorWrapper {
    type Target = TextureCreator<WindowContext>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Send for TextureCreatorWrapper {}
unsafe impl Sync for TextureCreatorWrapper {}

static TEXTURE_CREATOR: OnceLock<TextureCreatorWrapper> = OnceLock::new();

pub(crate) fn texture_creator() -> &'static TextureCreator<WindowContext> {
    &TEXTURE_CREATOR.get().unwrap()
}

pub struct Renderer {
    pub(crate) canvas: Canvas<Window>,
    pub(crate) images: HashMap<String, Texture<'static>>,
    pub(crate) atlas: Atlas,
}

impl Renderer {
    pub(crate) fn new(canvas: Canvas<Window>) -> Self {
        let tc = canvas.texture_creator();

        TEXTURE_CREATOR
            .set(TextureCreatorWrapper(tc))
            .map_err(|_| "Failed to set texture creator")
            .unwrap();

        let atlas = Atlas::new();

        Self {
            canvas,
            images: HashMap::new(),
            atlas,
        }
    }

    pub fn load_font<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P, size: f32) {
        let label = label.to_string();

        let bytes = std::fs::read(path).expect("Failed to read font file");
        let font = fontdue::Font::from_bytes(bytes.as_slice(), Default::default())
            .expect("Failed to load font");

        self.atlas.fonts.insert(
            label,
            (
                Font {
                    inner: Rc::new(font),
                    size,
                },
                HashMap::new(),
            ),
        );
    }

    pub fn load_image<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P) {
        let img = image::open(&path).expect("Failed to load image");
        let (width, height) = img.dimensions();
        let img = img.to_rgba8();

        let mut texture = texture_creator()
            .create_texture_target(PixelFormatEnum::RGBA32, width, height)
            .expect("Failed to create texture");

        texture
            .update(None, &img, (width * 4) as usize)
            .expect("Failed to update texture");

        texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        self.images.insert(label.to_string(), texture);
    }

    pub fn set_font<L: ToString>(&mut self, label: L) {
        self.atlas.current_font = label.to_string();
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub(crate) fn present(&mut self) {
        self.canvas.present();
    }

    pub fn color(&mut self) -> Color {
        self.canvas.draw_color()
    }

    pub fn set_color(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
    }

    pub fn draw_pixel<P: Into<Vec2>>(&mut self, pos: P) {
        self.canvas.draw_fpoint(FPoint::from(pos.into())).unwrap();
    }

    pub fn draw_pixels<P: Into<Vec2>, I: IntoIterator<Item = P>>(&mut self, pixels: I) {
        for pixel in pixels {
            self.draw_pixel(pixel);
        }
    }

    pub fn draw_line<P: Into<Vec2>, Q: Into<Vec2>>(&mut self, start: P, end: Q) {
        self.canvas
            .draw_fline(FPoint::from(start.into()), FPoint::from(end.into()))
            .unwrap();
    }

    pub fn draw_lines<P: Into<Vec2>, Q: Into<Vec2>, I: IntoIterator<Item = (P, Q)>>(
        &mut self,
        lines: I,
    ) {
        for (start, end) in lines {
            self.draw_line(start, end);
        }
    }

    pub fn draw_rect<P: Into<Vec2>, S: Into<Size>>(&mut self, pos: P, size: S) {
        let pos: Vec2 = pos.into();
        let size: Size = size.into();

        self.canvas
            .draw_frect(FRect::new(
                pos.x,
                pos.y,
                size.width as f32,
                size.height as f32,
            ))
            .unwrap();
    }

    pub fn draw_rects<P: Into<Vec2>, S: Into<Size>, I: IntoIterator<Item = (P, S)>>(
        &mut self,
        rects: I,
    ) {
        for (pos, size) in rects {
            self.draw_rect(pos, size);
        }
    }

    pub fn fill_rect<P: Into<Vec2>, S: Into<Size>>(&mut self, pos: P, size: S) {
        let pos: Vec2 = pos.into();
        let size: Size = size.into();

        self.canvas
            .fill_frect(FRect::new(
                pos.x,
                pos.y,
                size.width as f32,
                size.height as f32,
            ))
            .unwrap();
    }

    pub fn fill_rects<P: Into<Vec2>, S: Into<Size>, I: IntoIterator<Item = (P, S)>>(
        &mut self,
        rects: I,
    ) {
        for (pos, size) in rects {
            self.fill_rect(pos, size);
        }
    }

    pub fn draw_image<L: ToString, P: Into<Vec2>>(&mut self, label: L, pos: P) {
        let label = label.to_string();
        let pos: Vec2 = pos.into();

        if let Some(texture) = self.images.get(&label) {
            let query = texture.query();
            let dest = FRect::new(pos.x, pos.y, query.width as f32, query.height as f32);

            self.canvas.copy_f(texture, None, dest).unwrap();
        }
    }

    pub fn fill_text<T: ToString, P: Into<Vec2>>(&mut self, text: T, pos: P, color: Color) {
        let text = text.to_string();
        let pos: Vec2 = pos.into();

        let (r, g, b) = (color.r, color.g, color.b);

        let (font, _) = self.atlas.fonts.get(&self.atlas.current_font).unwrap();

        self.atlas.layout.append(
            &[font.inner.clone()],
            &TextStyle::new(text.as_str(), font.size as f32, 0),
        );

        let glyphs: Vec<_> = text
            .chars()
            .into_iter()
            .zip(
                self.atlas
                    .layout
                    .glyphs()
                    .into_iter()
                    .map(|g| (g.x, g.y, g.width, g.height)),
            )
            .collect();

        for (ch, (x, y, width, height)) in glyphs {
            if ch.is_whitespace() {
                continue;
            }

            if let Some(texture) = self.atlas.get_glyph(ch) {
                let dest = FRect::new(
                    pos.x + x as f32,
                    pos.y + y as f32,
                    width as f32,
                    height as f32,
                );

                texture.set_color_mod(r, g, b);

                self.canvas.copy_f(texture, None, dest).unwrap();
            } else {
                self.atlas.insert_glyph(ch);
            }
        }

        self.atlas.layout.clear();
    }
}
