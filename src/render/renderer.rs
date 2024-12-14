use super::{
    atlas::{Atlas, TextureKind},
    font::Font,
};
use crate::{
    math::{
        circles::{CircleFill, CircleOutline},
        Size, ToU32, Vec2,
    },
    traits::LoadSurface,
};
use fontdue::layout::TextStyle;
use sdl2::{
    pixels::Color,
    rect::{FPoint, FRect},
    render::{Canvas, Texture, TextureCreator},
    surface::Surface,
    video::{Window, WindowContext},
};
use std::{ops::Deref, path::Path, sync::OnceLock};

/// Little bit of a hack to get around the lifetime issues with the texture creator
/// This is a wrapper that impements Send and Sync, so we can store it as a static
/// OnceLock, so all the textures created with it can be stored as 'static.
///
/// Safe as long as the texture creator is only used in the main thread,
/// but the compiler doesnt allow it, we're so gucci
struct TextureCreatorWrapper(TextureCreator<WindowContext>);

impl Deref for TextureCreatorWrapper {
    type Target = TextureCreator<WindowContext>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// :3
unsafe impl Send for TextureCreatorWrapper {}
unsafe impl Sync for TextureCreatorWrapper {}

/// Static texture creator
/// All textures created with it are 'static
static TEXTURE_CREATOR: OnceLock<TextureCreatorWrapper> = OnceLock::new();

/// Helper function so we dont have to unwrap the texture creator every time
pub(crate) fn texture_creator() -> &'static TextureCreator<WindowContext> {
    &TEXTURE_CREATOR.get().unwrap()
}

/// Renderer
///
/// The renderer is the main struct that you will interact with to draw things on the screen.
/// It has methods to draw pixels, lines, rectangles, circles, images and text.
///
/// All the textures are stored in the atlas, so they can be cached and reused,
/// saving cpu and gpu time.
pub struct Renderer {
    /// Internal sdl2 canvas
    pub(crate) canvas: Canvas<Window>,
    /// Texture atlas
    pub(crate) atlas: Atlas,
}

impl Renderer {
    /// Creates a new renderer
    /// Only one renderer is allowed per window
    pub(crate) fn new(canvas: Canvas<Window>) -> Self {
        let tc = canvas.texture_creator();

        TEXTURE_CREATOR
            .set(TextureCreatorWrapper(tc))
            .map_err(|_| "Failed to set texture creator")
            .unwrap();

        let atlas = Atlas::new();

        Self { canvas, atlas }
    }

    /// Loads a font from a .ttf file.
    /// The font is stored in the atlas with the given label.
    pub fn load_font<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P, size: f32) {
        let label = label.to_string();

        let bytes = std::fs::read(path).expect("Failed to read font file");
        let font = fontdue::Font::from_bytes(bytes.as_slice(), Default::default())
            .expect("Failed to load font");

        self.atlas.fonts.insert(label, Font::new(font, size));
    }

    /// Loads an image from a file.
    /// The image is stored in the atlas with the given label.
    pub fn load_image<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P) {
        let label = label.to_string();
        let surface = Surface::from_file(path);

        let mut texture = texture_creator()
            .create_texture_from_surface(surface)
            .unwrap();

        texture.set_blend_mode(sdl2::render::BlendMode::Blend);

        self.atlas
            .insert_texture(&TextureKind::Image(label.into()), texture);
    }

    /// Sets the current font to be used when drawing text.
    pub fn set_font<L: ToString>(&mut self, label: L) {
        self.atlas.current_font = label.to_string();
    }

    /// Clears everything on the screen
    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    /// Internal function to say that we're finished drawing for this frame
    pub(crate) fn present(&mut self) {
        self.canvas.present();
    }

    /// Returns the current drawing color
    pub fn color(&mut self) -> Color {
        self.canvas.draw_color()
    }

    /// Sets the drawing color
    /// Everything drawn after this will be in this color
    pub fn set_color(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
    }

    /// Draws a pixel at the given position
    pub fn draw_pixel<P: Into<Vec2>>(&mut self, pos: P) {
        self.canvas.draw_fpoint(FPoint::from(pos.into())).unwrap();
    }

    /// Draws multiple pixels at the given positions
    /// Saves time by batching the draw calls
    pub fn draw_pixels<P: Into<Vec2>, I: IntoIterator<Item = P>>(&mut self, pixels: I) {
        let pixels = pixels
            .into_iter()
            .map(|pos| FPoint::from(pos.into()))
            .collect::<Vec<_>>();

        self.canvas.draw_fpoints(&*pixels).unwrap();
    }

    /// Draws a line from start to end
    pub fn draw_line<P: Into<Vec2>, Q: Into<Vec2>>(&mut self, start: P, end: Q) {
        self.canvas
            .draw_fline(FPoint::from(start.into()), FPoint::from(end.into()))
            .unwrap();
    }

    /// Draws multiple lines
    /// They are all connected, so the first point of the next line is the last point of the previous line
    /// Saves time by batching the draw calls
    pub fn draw_lines<P: Into<Vec2>, I: IntoIterator<Item = P>>(&mut self, lines: I) {
        let lines = lines
            .into_iter()
            .map(|pos| FPoint::from(pos.into()))
            .collect::<Vec<_>>();

        self.canvas.draw_flines(&*lines).unwrap();
    }

    /// Draws the outline of a rectangle
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

    /// Draws multiple rectangles
    /// Saves time by batching the draw calls
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

    /// Draws a filled rectangle
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

    /// Draws multiple filled rectangles
    /// Saves time by batching the draw calls
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

    /// Draws an arc
    /// The arc is drawn from start_angle to end_angle
    pub fn draw_arc<P: Into<Vec2>, U: ToU32>(
        &mut self,
        center: P,
        radius: U,
        start_angle: f32,
        end_angle: f32,
    ) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2;
        let color = self.color();

        let kind = TextureKind::Arc(radius, start_angle as i32, end_angle as i32);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(&kind) {
                texture
            } else {
                let texture = Texture::arc(texture_creator(), radius, start_angle, end_angle);

                self.atlas.insert_texture(&kind, texture);
                self.atlas.get_texture(&kind).unwrap()
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

    /// Draws a pixelated circle outline
    pub fn draw_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2;
        let color = self.color();

        let kind = TextureKind::Circle(radius);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(&kind) {
                texture
            } else {
                let texture = Texture::circle_outline(texture_creator(), radius);

                self.atlas.insert_texture(&kind, texture);
                self.atlas.get_texture(&kind).unwrap()
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

    /// Draws an anti-aliased (smooth) circle outline
    pub fn draw_aa_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2;
        let color = self.color();

        let kind = TextureKind::AACircle(radius);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(&kind) {
                texture
            } else {
                let texture = Texture::aa_circle_outline(texture_creator(), radius);
                self.atlas.insert_texture(&kind, texture);
                self.atlas.get_texture(&kind).unwrap()
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

    /// Draws a filled pixelated circle
    pub fn fill_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center = center.into();
        let radius = radius.to_u32();
        let diameter = radius * 2;
        let color = self.color();

        let kind = TextureKind::FilledCircle(radius);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(&kind) {
                texture
            } else {
                let texture = Texture::circle_fill(texture_creator(), radius);

                self.atlas.insert_texture(&kind, texture);
                self.atlas.get_texture(&kind).unwrap()
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

    /// Draws a filled anti-aliased (smooth) circle
    pub fn fill_aa_circle<P: Into<Vec2>, U: ToU32>(&mut self, center: P, radius: U) {
        let center: Vec2 = center.into();
        let radius: u32 = radius.to_u32();
        let diameter = radius * 2;
        let color = self.color();

        let kind = TextureKind::AAFilledCircle(radius);

        let texture = if let Some(texture) = self.atlas.get_texture(&kind) {
            texture
        } else {
            let texture = Texture::aa_circle_fill(texture_creator(), radius);

            self.atlas.insert_texture(&kind, texture);
            self.atlas.get_texture(&kind).unwrap()
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

    /// Draws an image at the given position
    /// The label is the name of the image that was loaded with `load_image`
    ///
    /// If not found, it does nothing
    pub fn draw_image<L: ToString, P: Into<Vec2>>(&mut self, label: L, pos: P) {
        let label = label.to_string();
        let pos: Vec2 = pos.into();

        let kind = TextureKind::Image(label.into());

        if let Some(texture) = self.atlas.get_texture(&kind) {
            let query = texture.query();
            let dest = FRect::new(pos.x, pos.y, query.width as f32, query.height as f32);

            self.canvas.copy_f(texture, None, dest).unwrap();
        }
    }

    /// Draws text at the given position
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

    /// Returns the size of the given text
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
