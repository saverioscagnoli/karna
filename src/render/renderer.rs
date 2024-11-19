use super::cache::{TextureCache, TextureKind};
use crate::{
    math::{Size, Vec2},
    traits::ToU32,
};
use anyhow::anyhow;
use sdl2::{
    pixels::{Color, PixelFormatEnum},
    rect::{FPoint, FRect},
    render::{BlendMode, Canvas, TextureCreator},
    ttf::Font,
    video::{Window, WindowContext},
};
use std::{collections::HashMap, path::Path, rc::Rc, sync::OnceLock};

struct TextureCreatorWrapper(TextureCreator<WindowContext>);

unsafe impl Send for TextureCreatorWrapper {}
unsafe impl Sync for TextureCreatorWrapper {}

static TTF_CTX: OnceLock<sdl2::ttf::Sdl2TtfContext> = OnceLock::new();
static TEXTURE_CREATOR: OnceLock<TextureCreatorWrapper> = OnceLock::new();

fn ttf() -> &'static sdl2::ttf::Sdl2TtfContext {
    &TTF_CTX.get_or_init(|| sdl2::ttf::init().map_err(|e| anyhow!(e)).unwrap())
}

fn texture_creator() -> &'static TextureCreator<WindowContext> {
    &TEXTURE_CREATOR
        .get_or_init(|| {
            panic!("Failed to get texture creator.");
        })
        .0
}

pub struct Renderer {
    canvas: Canvas<Window>,
    pub(crate) cache: TextureCache,
    fonts: HashMap<String, Rc<Font<'static, 'static>>>,
    font: Option<Rc<Font<'static, 'static>>>,
}

impl Renderer {
    pub(crate) fn new(canvas: Canvas<Window>) -> Self {
        TEXTURE_CREATOR
            .set(TextureCreatorWrapper(canvas.texture_creator()))
            .map_err(|_| anyhow!("Failed to set texture creator."))
            .unwrap();

        Self {
            canvas,
            cache: TextureCache::new(),
            fonts: HashMap::new(),
            font: None,
        }
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub(crate) fn present(&mut self) {
        self.canvas.present();
    }

    pub fn set_logical_size<S: Into<Size>>(&mut self, size: S) {
        let size = size.into();

        self.canvas
            .set_logical_size(size.width, size.height)
            .unwrap();
    }

    pub fn draw_color(&self) -> Color {
        self.canvas.draw_color()
    }

    pub fn set_color(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
    }

    /// Loads a font from a .ttf file.
    /// For the compiled executable to work, the font file must exist
    /// at the time of execution, as a separate file.
    ///
    /// To include the font file in the compiled executable, use the
    /// `include_font` method instead.
    pub fn load_font<T: ToString, P: AsRef<Path>>(&mut self, label: T, path: P, size: u16) {
        let path = path.as_ref();
        let label = label.to_string();

        let font = Rc::new(ttf().load_font(path, size).map_err(|e| anyhow!(e)).unwrap());

        self.fonts.insert(label, font);
    }

    /// Loads a font and includes it in the compiled executable.
    /// This is useful when font files are relatively small and
    /// can be included in the executable.
    ///
    /// # Example
    /// const FONT_DATA: &[u8] = include_bytes!("path/to/font.ttf");
    ///
    /// ctx.render.include_font("default", FONT_DATA, 20);
    pub fn include_font<T: ToString>(&mut self, label: T, font_data: &'static [u8], size: u16) {
        let label = label.to_string();

        let rwops = sdl2::rwops::RWops::from_bytes(font_data).unwrap();
        let font = ttf()
            .load_font_from_rwops(rwops, size)
            .map_err(|_| anyhow!("Failed to load font: {}", label))
            .unwrap();

        let font = Rc::new(font);

        self.fonts.insert(label, font);
    }

    pub fn set_font<T: ToString>(&mut self, label: T) {
        let label = label.to_string();
        let font = self
            .fonts
            .get(&label)
            .ok_or_else(|| anyhow!("Font not found."))
            .unwrap();

        self.font = Some(font.clone());
    }

    pub fn draw_pixel<P: Into<Vec2>>(&mut self, pos: P) {
        let pos = pos.into();

        self.canvas.draw_fpoint(FPoint::new(pos.x, pos.y)).unwrap();
    }

    pub fn draw_pixels<P: Into<Vec2>, I: IntoIterator<Item = P>>(&mut self, points: I) {
        let points: Vec<FPoint> = points
            .into_iter()
            .map(|p| {
                let p = p.into();
                FPoint::new(p.x, p.y)
            })
            .collect();

        self.canvas.draw_fpoints(&*points).unwrap();
    }

    pub fn draw_line<P1: Into<Vec2>, P2: Into<Vec2>>(&mut self, p1: P1, p2: P2) {
        let p1 = p1.into();
        let p2 = p2.into();

        self.canvas
            .draw_fline(FPoint::new(p1.x, p1.y), FPoint::new(p2.x, p2.y))
            .unwrap();
    }

    pub fn draw_lines<P: Into<Vec2>, I: IntoIterator<Item = P>>(&mut self, points: I) {
        let points: Vec<FPoint> = points
            .into_iter()
            .map(|p| {
                let p = p.into();
                FPoint::new(p.x, p.y)
            })
            .collect();

        self.canvas.draw_flines(&*points).unwrap();
    }

    pub fn draw_rect<P: Into<Vec2>, S: Into<Size>>(&mut self, pos: P, size: S) {
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

    pub fn draw_rects<P: Into<Vec2>, S: Into<Size>, I: IntoIterator<Item = (P, S)>>(
        &mut self,
        rects: I,
    ) {
        let rects: Vec<FRect> = rects
            .into_iter()
            .map(|(pos, size)| {
                let pos = pos.into();
                let size = size.into();

                FRect::new(pos.x, pos.y, size.width as f32, size.height as f32)
            })
            .collect();

        self.canvas.draw_frects(&*rects).unwrap();
    }

    pub fn fill_rect<P: Into<Vec2>, S: Into<Size>>(&mut self, pos: P, size: S) {
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

    pub fn fill_rects<P: Into<Vec2>, S: Into<Size>, I: IntoIterator<Item = (P, S)>>(
        &mut self,
        rects: I,
    ) {
        let rects: Vec<FRect> = rects
            .into_iter()
            .map(|(pos, size)| {
                let pos = pos.into();
                let size = size.into();

                FRect::new(pos.x, pos.y, size.width as f32, size.height as f32)
            })
            .collect();

        self.canvas.fill_frects(&*rects).unwrap();
    }

    fn set_pixel_color(buffer: &mut [u8], i: usize, color: Color) {
        #[cfg(target_endian = "big")]
        {
            buffer[i] = color.r;
            buffer[i + 1] = color.g;
            buffer[i + 2] = color.b;
            buffer[i + 3] = color.a;
        }

        #[cfg(target_endian = "little")]
        {
            buffer[i] = color.a;
            buffer[i + 1] = color.b;
            buffer[i + 2] = color.g;
            buffer[i + 3] = color.r;
        }
    }

    fn blend_pixel_color(buffer: &mut [u8], index: usize, color: Color, alpha: f32) {
        let inv_alpha = 1.0 - alpha;

        Self::set_pixel_color(
            buffer,
            index,
            Color {
                r: (color.r as f32 * alpha + buffer[index] as f32 * inv_alpha) as u8,
                g: (color.g as f32 * alpha + buffer[index + 1] as f32 * inv_alpha) as u8,
                b: (color.b as f32 * alpha + buffer[index + 2] as f32 * inv_alpha) as u8,
                a: (color.a as f32 * alpha + buffer[index + 3] as f32 * inv_alpha) as u8,
            },
        );
    }

    pub fn draw_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2 + 1;
        let color = self.canvas.draw_color();

        let kind = TextureKind::CircleOutline(radius, color);

        let texture = match self.cache.get(&kind) {
            Some(texture) => texture,
            None => {
                let mut texture = texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGBA8888, diameter, diameter)
                    .unwrap();

                texture.set_blend_mode(BlendMode::Blend);

                texture
                    .with_lock(None, |buffer, _| {
                        let mut t1 = (radius / 16) as i32;
                        let mut t2;
                        let mut x = radius as i32;
                        let mut y = 0 as i32;

                        let center = radius as i32;

                        while x >= y {
                            let points = [
                                (x, y),
                                (y, x),
                                (-y, x),
                                (-x, y),
                                (-x, -y),
                                (-y, -x),
                                (y, -x),
                                (x, -y),
                            ];

                            for (x, y) in points.iter() {
                                let dx = center + x;
                                let dy = center + y;

                                let i = (dy * diameter as i32 + dx) as usize * 4;

                                Self::set_pixel_color(buffer, i, color);
                            }

                            y += 1;
                            t1 += y;
                            t2 = t1 - x;

                            if t2 >= 0 {
                                t1 = t2;
                                x -= 1;
                            }
                        }
                    })
                    .unwrap();

                self.cache.insert(&kind, texture);
                self.cache.get(&kind).unwrap()
            }
        };

        let query = texture.query();

        self.canvas
            .copy_f(
                texture,
                None,
                Some(FRect::new(
                    center.x - radius as f32,
                    center.y - radius as f32,
                    query.width as f32,
                    query.height as f32,
                )),
            )
            .unwrap();
    }

    pub fn fill_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2 + 1;
        let color = self.canvas.draw_color();

        let kind = TextureKind::CircleFilled(radius, color);

        let texture = match self.cache.get(&kind) {
            Some(texture) => texture,
            None => {
                let mut texture = texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGBA8888, diameter, diameter)
                    .unwrap();

                texture.set_blend_mode(BlendMode::Blend);

                texture
                    .with_lock(None, |buffer, _| {
                        let mut t1 = (radius / 16) as i32;
                        let mut t2;
                        let mut x = radius as i32;
                        let mut y = 0 as i32;

                        let center = radius as i32;

                        while x >= y {
                            for dy in -y..y {
                                let x1 = center - x;
                                let x2 = center + x;

                                for dx in x1..x2 {
                                    let i = (dy + center) * diameter as i32 + dx;
                                    Self::set_pixel_color(buffer, i as usize * 4, color);
                                }
                            }

                            for dy in -x..x {
                                let y1 = center - y;
                                let y2 = center + y;

                                for dx in y1..y2 {
                                    let nx = dx;
                                    let ny = center + dy;

                                    let i = ny * diameter as i32 + nx;
                                    Self::set_pixel_color(buffer, i as usize * 4, color);
                                }
                            }

                            y += 1;
                            t1 += y;
                            t2 = t1 - x;

                            if t2 >= 0 {
                                t1 = t2;
                                x -= 1;
                            }
                        }
                    })
                    .unwrap();

                self.cache.insert(&kind, texture);
                self.cache.get(&kind).unwrap()
            }
        };

        let query = texture.query();

        self.canvas
            .copy_f(
                texture,
                None,
                Some(FRect::new(
                    center.x - radius as f32,
                    center.y - radius as f32,
                    query.width as f32,
                    query.height as f32,
                )),
            )
            .unwrap();
    }

    pub fn fill_aa_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2 + 1;
        let color = self.canvas.draw_color();

        let kind = TextureKind::CircleFilledAA(radius, color);

        let texture = match self.cache.get(&kind) {
            Some(texture) => texture,
            None => {
                let mut texture = texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGBA8888, diameter, diameter)
                    .unwrap();

                texture.set_blend_mode(BlendMode::Blend);

                texture
                    .with_lock(None, |buffer, _| {
                        let center = radius as i32;
                        let samples = 4;

                        for y in 0..diameter {
                            for x in 0..diameter {
                                let mut alpha_sum = 0.0;

                                for sy in 0..samples {
                                    for sx in 0..samples {
                                        let sub_x = x as f32 + (sx as f32 + 0.5) / samples as f32;
                                        let sub_y = y as f32 + (sy as f32 + 0.5) / samples as f32;
                                        let dx = sub_x - center as f32;
                                        let dy = sub_y - center as f32;
                                        let distance = (dx * dx + dy * dy).sqrt();

                                        let alpha = if distance < radius as f32 {
                                            1.0
                                        } else if distance < radius as f32 + 1.0 {
                                            1.0 - (distance - radius as f32)
                                        } else {
                                            0.0
                                        };
                                        alpha_sum += alpha;
                                    }
                                }

                                let alpha = alpha_sum / (samples * samples) as f32;

                                if alpha > 0.0 {
                                    let i = ((y as u32 * diameter + x as u32) * 4) as usize;
                                    Self::blend_pixel_color(buffer, i, color, alpha);
                                }
                            }
                        }
                    })
                    .unwrap();

                self.cache.insert(&kind, texture);
                self.cache.get(&kind).unwrap()
            }
        };

        let query = texture.query();

        self.canvas
            .copy_f(
                texture,
                None,
                Some(FRect::new(
                    center.x - radius as f32,
                    center.y - radius as f32,
                    query.width as f32,
                    query.height as f32,
                )),
            )
            .unwrap();
    }

    pub fn fill_text<T: ToString, P: Into<Vec2>>(&mut self, text: T, pos: P, color: Color) {
        let text: Rc<str> = text.to_string().into();
        let mut pos = pos.into();

        let font = self.font.as_ref().unwrap();

        let kind = TextureKind::Text(text.clone(), color);

        if let Some(texture) = self.cache.get(&kind) {
            let query = texture.query();

            self.canvas
                .copy_f(
                    texture,
                    None,
                    Some(FRect::new(
                        pos.x,
                        pos.y,
                        query.width as f32,
                        query.height as f32,
                    )),
                )
                .unwrap();

            return;
        }

        for ch in text.chars() {
            let kind = TextureKind::Char(ch, color);

            let texture = match self.cache.get(&kind) {
                Some(texture) => texture,
                None => {
                    let surface = font.render_char(ch).blended(color).unwrap();
                    let texture = texture_creator()
                        .create_texture_from_surface(&surface)
                        .map_err(|e| anyhow!(e))
                        .unwrap();

                    self.cache.insert(&kind, texture);
                    self.cache.get(&kind).unwrap()
                }
            };

            let query = texture.query();

            self.canvas
                .copy_f(
                    texture,
                    None,
                    Some(FRect::new(
                        pos.x,
                        pos.y,
                        query.width as f32,
                        query.height as f32,
                    )),
                )
                .unwrap();

            pos.x += query.width as f32;
        }

        // TODO: at this point all the characters should have
        // ben joined into a single texture, so we can cache it.
        //self.cache.insert(&kind, texture);
    }
}
