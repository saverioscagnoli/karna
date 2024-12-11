use super::{
    atlas::{Atlas, TextureKind},
    font::Font,
};
use crate::{
    math::{Size, ToU32, Vec2},
    traits::LoadSurface,
};
use fontdue::layout::TextStyle;
use sdl2::{
    pixels::{Color, PixelFormatEnum},
    rect::{FPoint, FRect},
    render::{BlendMode, Canvas, Texture, TextureCreator},
    surface::Surface,
    video::{Window, WindowContext},
};
use std::{collections::HashMap, ops::Deref, path::Path, sync::OnceLock};

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

    pub fn load_font<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P, size: f32) {
        let label = label.to_string();

        let bytes = std::fs::read(path).expect("Failed to read font file");
        let font = fontdue::Font::from_bytes(bytes.as_slice(), Default::default())
            .expect("Failed to load font");

        self.atlas.fonts.insert(label, Font::new(font, size));
    }

    pub fn load_image<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P) {
        let surface = Surface::from_file(path);

        let mut texture = texture_creator()
            .create_texture_from_surface(surface)
            .unwrap();

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
        let pixels = pixels
            .into_iter()
            .map(|pos| FPoint::from(pos.into()))
            .collect::<Vec<_>>();

        self.canvas.draw_fpoints(&*pixels).unwrap();
    }

    pub fn draw_line<P: Into<Vec2>, Q: Into<Vec2>>(&mut self, start: P, end: Q) {
        self.canvas
            .draw_fline(FPoint::from(start.into()), FPoint::from(end.into()))
            .unwrap();
    }

    pub fn draw_lines<P: Into<Vec2>, I: IntoIterator<Item = P>>(&mut self, lines: I) {
        let lines = lines
            .into_iter()
            .map(|pos| FPoint::from(pos.into()))
            .collect::<Vec<_>>();

        self.canvas.draw_flines(&*lines).unwrap();
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
        let rects: Vec<FRect> = rects
            .into_iter()
            .map(|(pos, size)| {
                let pos: Vec2 = pos.into();
                let size: Size = size.into();

                FRect::new(pos.x, pos.y, size.width as f32, size.height as f32)
            })
            .collect();

        self.canvas.draw_frects(&rects).unwrap();
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
        let rects: Vec<FRect> = rects
            .into_iter()
            .map(|(pos, size)| {
                let pos: Vec2 = pos.into();
                let size: Size = size.into();

                FRect::new(pos.x, pos.y, size.width as f32, size.height as f32)
            })
            .collect();

        self.canvas.fill_frects(&rects).unwrap();
    }

    pub fn draw_arc<P: Into<Vec2>, U: ToU32>(
        &mut self,
        center: P,
        radius: U,
        start_angle: f32,
        end_angle: f32,
    ) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2 + 1;
        let color = self.color();

        let kind = TextureKind::Arc(radius, start_angle as i32, end_angle as i32);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(kind) {
                texture
            } else {
                let mut texture = texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGBA32, diameter, diameter)
                    .unwrap();

                texture.set_blend_mode(BlendMode::Blend);

                texture
                    .with_lock(None, |buffer, _| {
                        let mut t1 = (radius / 16) as i32;
                        let mut t2;
                        let mut x = radius as i32;
                        let mut y = 0 as i32;

                        let center = radius as i32;

                        let start_angle = start_angle.to_radians();
                        let end_angle = end_angle.to_radians();

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

                            for &(px, py) in points.iter() {
                                let dx = center + px;
                                let dy = center + py;

                                // Calculate angle of the point
                                let angle = (py as f32).atan2(px as f32);
                                let normalized_angle = if angle < 0.0 {
                                    angle + 2.0 * std::f32::consts::PI
                                } else {
                                    angle
                                };

                                // Check if the angle is within the range
                                if normalized_angle >= start_angle && normalized_angle <= end_angle
                                {
                                    let i = (dy * diameter as i32 + dx) as usize * 4;
                                    Self::set_pixel_color(buffer, i, Color::WHITE);
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

                self.atlas.insert_texture(kind, texture);
                self.atlas.get_texture(kind).unwrap()
            }
        };

        texture.set_color_mod(color.r, color.g, color.b);

        let dst = FRect::new(
            center.x - radius as f32,
            center.y - radius as f32,
            diameter as f32,
            diameter as f32,
        );

        self.canvas.copy_f(texture, None, dst).unwrap();
    }

    pub fn draw_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2 + 1;
        let color = self.color();

        let kind = TextureKind::Circle(radius);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(kind) {
                texture
            } else {
                let mut texture = texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGBA32, diameter, diameter)
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

                                Self::set_pixel_color(buffer, i, Color::WHITE);
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

                self.atlas.insert_texture(kind, texture);
                self.atlas.get_texture(kind).unwrap()
            }
        };

        texture.set_color_mod(color.r, color.g, color.b);

        let dst = FRect::new(
            center.x - radius as f32,
            center.y - radius as f32,
            diameter as f32,
            diameter as f32,
        );

        self.canvas.copy_f(texture, None, dst).unwrap();
    }

    pub fn draw_aa_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2 + 1;
        let color = self.color();

        let kind = TextureKind::AACircle(radius);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(kind) {
                texture
            } else {
                let mut texture = texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGBA32, diameter, diameter)
                    .unwrap();

                texture.set_blend_mode(BlendMode::Blend);

                texture
                    .with_lock(None, |buffer, _| {
                        let outline_thickness = 4.0f32;
                        // Move the center otherwise the circle will be cut off
                        let center = radius as f32 + 0.5;
                        let samples = 4;

                        for y in 0..diameter {
                            for x in 0..diameter {
                                let mut alpha_sum = 0.0;

                                for sy in 0..samples {
                                    for sx in 0..samples {
                                        let sub_x = x as f32 + (sx as f32 + 0.5) / samples as f32;
                                        let sub_y = y as f32 + (sy as f32 + 0.5) / samples as f32;
                                        let dx = sub_x - center;
                                        let dy = sub_y - center;
                                        let distance = (dx * dx + dy * dy).sqrt();

                                        let inner_edge = radius as f32 - outline_thickness / 2.0;
                                        let outer_edge = radius as f32 + outline_thickness / 2.0;

                                        let alpha =
                                            if distance >= inner_edge && distance <= outer_edge {
                                                1.0 - ((distance - radius as f32).abs()
                                                    / (outline_thickness / 2.0))
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

                self.atlas.insert_texture(kind, texture);
                self.atlas.get_texture(kind).unwrap()
            }
        };

        texture.set_color_mod(color.r, color.g, color.b);

        let dst = FRect::new(
            center.x - radius as f32,
            center.y - radius as f32,
            diameter as f32,
            diameter as f32,
        );

        self.canvas.copy_f(texture, None, dst).unwrap();
    }

    pub fn fill_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2 + 1;
        let color = self.color();

        let kind = TextureKind::FilledCircle(radius);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(kind) {
                texture
            } else {
                let mut texture = texture_creator()
                    .create_texture_streaming(PixelFormatEnum::RGBA32, diameter, diameter)
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
                                    Self::set_pixel_color(buffer, i as usize * 4, Color::WHITE);
                                }
                            }

                            for dy in -x..x {
                                let y1 = center - y;
                                let y2 = center + y;

                                for dx in y1..y2 {
                                    let nx = dx;
                                    let ny = center + dy;

                                    let i = ny * diameter as i32 + nx;
                                    Self::set_pixel_color(buffer, i as usize * 4, Color::WHITE);
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

                self.atlas.insert_texture(kind, texture);
                self.atlas.get_texture(kind).unwrap()
            }
        };

        texture.set_color_mod(color.r, color.g, color.b);

        let dst = FRect::new(
            center.x - radius as f32,
            center.y - radius as f32,
            diameter as f32,
            diameter as f32,
        );

        self.canvas.copy_f(texture, None, dst).unwrap();
    }

    pub fn fill_aa_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center: Vec2 = center.into();
        let radius: u32 = radius.to_u32();
        let diameter = radius * 2 + 1;
        let color = self.color();

        let kind = TextureKind::AAFilledCircle(radius);

        let texture = if let Some(texture) = self.atlas.get_texture(kind) {
            texture
        } else {
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

            self.atlas.insert_texture(kind, texture);
            self.atlas.get_texture(kind).unwrap()
        };

        texture.set_color_mod(color.r, color.g, color.b);

        let dst = FRect::new(
            center.x - radius as f32,
            center.y - radius as f32,
            diameter as f32,
            diameter as f32,
        );

        self.canvas.copy_f(texture, None, dst).unwrap();
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

        let glyphs = {
            let font = self.atlas.fonts.get_mut(&self.atlas.current_font).unwrap();

            font.layout.append(
                &[font.inner.clone()],
                &TextStyle::new(text.as_str(), font.size as f32, 0),
            );

            let glyphs = text
                .chars()
                .into_iter()
                .zip(
                    font.layout
                        .glyphs()
                        .into_iter()
                        .map(|g| (g.x, g.y, g.width, g.height)),
                )
                .collect::<Vec<_>>();

            font.layout.clear();

            glyphs
        };

        // Process glyphs after font borrow is dropped
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
    }

    pub fn text_size<T: ToString>(&mut self, text: T) -> Size {
        let text = text.to_string();

        let font = self.atlas.fonts.get_mut(&self.atlas.current_font).unwrap();

        font.layout.append(
            &[font.inner.clone()],
            &TextStyle::new(text.as_str(), font.size as f32, 0),
        );

        let mut min_left = 0.0;
        let mut min_top = 0.0;
        let mut max_right = 0.0;
        let mut max_bottom = 0.0;

        for glyph in font.layout.glyphs() {
            let left = glyph.x;
            let top = glyph.y;
            let right = glyph.x + glyph.width as f32;
            let bottom = glyph.y + glyph.height as f32;

            if left < min_left {
                min_left = left;
            }
            if top < min_top {
                min_top = top;
            }
            if right > max_right {
                max_right = right;
            }
            if bottom > max_bottom {
                max_bottom = bottom;
            }
        }

        font.layout.clear();

        let width = (max_right - min_left) as u32;
        let height = (max_bottom - min_top) as u32;

        (width, height).into()
    }
}
