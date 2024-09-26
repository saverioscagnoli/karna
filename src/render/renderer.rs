use sdl2::{
    pixels::PixelFormatEnum,
    rect::FRect,
    render::{BlendMode, Canvas, TextureCreator},
    ttf::{Font, Sdl2TtfContext},
    video::{Window, WindowContext},
};
use std::{cell::OnceCell, collections::HashMap, path::Path};

use super::{
    cache::{TextureCache, TextureType},
    Color,
};
use crate::{
    math::{self, Size, ToU32, Vector2},
    throw,
};

static mut TTF_CTX: OnceCell<sdl2::ttf::Sdl2TtfContext> = OnceCell::new();
static mut FONTS: OnceCell<HashMap<String, Font>> = OnceCell::new();
static mut TEXTURE_CREATOR: OnceCell<TextureCreator<WindowContext>> = OnceCell::new();
static mut TEXTURE_CACHE: OnceCell<TextureCache> = OnceCell::new();

fn ttf_ctx() -> &'static Sdl2TtfContext {
    unsafe { TTF_CTX.get().unwrap() }
}

fn fonts() -> &'static HashMap<String, Font<'static, 'static>> {
    unsafe { FONTS.get().unwrap() }
}

fn fonts_mut() -> &'static mut HashMap<String, Font<'static, 'static>> {
    unsafe { FONTS.get_mut().unwrap() }
}

pub fn load_font<L, P>(label: L, path: P, size: u16)
where
    L: ToString,
    P: AsRef<Path>,
{
    ttf_ctx()
        .load_font(path, size)
        .map(|font| {
            fonts_mut().insert(label.to_string(), font);
        })
        .unwrap();
}

fn texture_creator() -> &'static TextureCreator<WindowContext> {
    unsafe { TEXTURE_CREATOR.get().unwrap() }
}

fn cache_mut() -> &'static mut TextureCache {
    unsafe { TEXTURE_CACHE.get_mut().unwrap() }
}

pub(crate) fn init(texture_creator: TextureCreator<WindowContext>) {
    unsafe {
        TTF_CTX
            .set(sdl2::ttf::init().unwrap())
            .map_err(|_| {
                throw!(crate::Error::Sdl(
                    "Failed to initialize the TTF context.".to_string()
                ))
            })
            .unwrap();

        FONTS
            .set(HashMap::new())
            .map_err(|_| {
                throw!(crate::Error::Sdl(
                    "Failed to initialize the fonts.".to_string()
                ))
            })
            .unwrap();

        TEXTURE_CREATOR
            .set(texture_creator)
            .map_err(|_| {
                throw!(crate::Error::Sdl(
                    "Failed to initialize the texture creator.".to_string()
                ))
            })
            .unwrap();

        TEXTURE_CACHE
            .set(TextureCache::new())
            .map_err(|_| {
                throw!(crate::Error::Sdl(
                    "Failed to initialize the texture cache.".to_string()
                ))
            })
            .unwrap();
    }
}

pub struct Renderer {
    canvas: Canvas<Window>,
    font: String,
}

impl Renderer {
    pub fn new(canvas: Canvas<Window>) -> Self {
        Self {
            canvas,
            font: "".to_string(),
        }
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub(crate) fn present(&mut self) {
        self.canvas.present();
    }

    pub fn set_font<L>(&mut self, label: L)
    where
        L: ToString,
    {
        self.font = label.to_string();
    }

    pub fn set_color(&mut self, color: Color) {
        self.canvas
            .set_draw_color::<sdl2::pixels::Color>(color.into())
    }

    pub fn draw_pixel<P>(&mut self, pos: P)
    where
        P: Into<Vector2>,
    {
        let pos = pos.into();
        self.canvas.draw_fpoint(pos).unwrap();
    }

    pub fn draw_line<P1, P2>(&mut self, start: P1, end: P2)
    where
        P1: Into<Vector2>,
        P2: Into<Vector2>,
    {
        let start = start.into();
        let end = end.into();

        self.canvas.draw_fline(start, end).unwrap();
    }

    pub fn draw_rect<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<Vector2>,
        S: Into<Size>,
    {
        let pos = pos.into();
        let size = size.into();

        self.canvas
            .draw_frect(FRect::new(
                pos.x,
                pos.y,
                size.width as f32,
                size.height as f32,
            ))
            .unwrap();
    }

    pub fn fill_rect<P, S>(&mut self, pos: P, size: S)
    where
        P: Into<Vector2>,
        S: Into<Size>,
    {
        let pos = pos.into();
        let size = size.into();

        self.canvas
            .fill_frect(FRect::new(
                pos.x,
                pos.y,
                size.width as f32,
                size.height as f32,
            ))
            .unwrap();
    }

    pub fn draw_circle<P, U>(&mut self, center: P, radius: U)
    where
        P: Into<Vector2>,
        U: ToU32,
    {
        let center = center.into();
        let r = radius.to_u32();
        let d = r * 2 + 1;
        let color = self.canvas.draw_color();

        let texture = cache_mut().get_or_insert(TextureType::CircleOutline(r, color), || {
            let mut texture = texture_creator()
                .create_texture_streaming(PixelFormatEnum::RGBA8888, d, d)
                .unwrap();

            texture.set_blend_mode(BlendMode::Blend);

            texture
                .with_lock(None, |buffer, _| {
                    math::utils::fill_midpoint_circle_buffer(buffer, r, color);
                })
                .unwrap();

            texture
        });

        self.canvas
            .copy_f(
                texture,
                None,
                FRect::new(center.x - r as f32, center.y - r as f32, d as f32, d as f32),
            )
            .unwrap();
    }

    pub fn fill_circle<P, U>(&mut self, center: P, radius: U)
    where
        P: Into<Vector2>,
        U: ToU32,
    {
        let center = center.into();
        let r = radius.to_u32();
        let d = r * 2 + 1;
        let color = self.canvas.draw_color();

        let texture = cache_mut().get_or_insert(TextureType::CircleFill(r, color), || {
            let mut texture = texture_creator()
                .create_texture_streaming(PixelFormatEnum::RGBA8888, d, d)
                .unwrap();

            texture.set_blend_mode(BlendMode::Blend);

            texture
                .with_lock(None, |buffer, _| {
                    math::utils::fill_midpoint_circle_filled_buffer(buffer, r, color);
                })
                .unwrap();

            texture
        });

        self.canvas
            .copy_f(
                texture,
                None,
                FRect::new(center.x - r as f32, center.y - r as f32, d as f32, d as f32),
            )
            .unwrap();
    }

    pub fn fill_text<P, T>(&mut self, pos: P, text: T, color: Color)
    where
        P: Into<Vector2>,
        T: ToString,
    {
        let mut pos = pos.into();
        let text = text.to_string();
        let color = color.into();

        let font = fonts()
            .get(&self.font)
            .or_else(|| {
                if self.font == "" {
                    throw!(crate::Error::Render(
                        "No font set. Did you forget to call `renderer.set_font()`?".to_string()
                    ));
                } else {
                    throw!(crate::Error::Render(format!(
                        "Font with label '{}' not found.",
                        self.font
                    )))
                }
            })
            .unwrap();

        for c in text.chars() {
            let key = TextureType::Text(c.to_string(), color);

            let texture = cache_mut().get_or_insert(key, || {
                let surface = font
                    .render_char(c)
                    .blended::<sdl2::pixels::Color>(color)
                    .unwrap();

                texture_creator()
                    .create_texture_from_surface(&surface)
                    .unwrap()
            });

            let (width, height) = font.size_of_char(c).unwrap();

            self.canvas
                .copy_f(
                    texture,
                    None,
                    FRect::new(pos.x, pos.y, width as f32, height as f32),
                )
                .unwrap();

            pos.x += width as f32;
        }
    }
}
