use super::{
    atlas::{Atlas, TextureKind},
    font::Font,
    shaders::create_shader_program,
};
use crate::{
    math::{
        circles::{CircleFill, CircleOutline},
        Size, ToU32, Vec2,
    },
    traits::LoadSurface,
};
use fontdue::layout::TextStyle;
use gl::types::{GLint, GLsizei};
use hashbrown::HashMap;
use sdl2::{
    pixels::PixelFormatEnum,
    rect::{FPoint, FRect},
    render::{Canvas, Texture, TextureCreator},
    surface::{Surface, SurfaceContext, SurfaceRef},
};
use std::{ops::Deref, path::Path, rc::Rc, sync::OnceLock};

pub use sdl2::pixels::Color;

struct TC(TextureCreator<SurfaceContext<'static>>);

unsafe impl Send for TC {}
unsafe impl Sync for TC {}

impl Deref for TC {
    type Target = TextureCreator<SurfaceContext<'static>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

static TEXTURE_CREATOR: OnceLock<TC> = OnceLock::new();

pub(crate) fn texture_creator() -> &'static TextureCreator<SurfaceContext<'static>> {
    TEXTURE_CREATOR.get().unwrap()
}

pub struct Renderer {
    pub(crate) texture_id: u32,
    pub(crate) canvas: Canvas<Surface<'static>>,
    pub(crate) atlas: Atlas,
    pub(crate) shaders: HashMap<String, u32>,
    active_shader: (Rc<str>, u32),
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        let canvas = Surface::new(width, height, PixelFormatEnum::RGBA32)
            .unwrap()
            .into_canvas()
            .unwrap();

        TEXTURE_CREATOR
            .set(TC(canvas.texture_creator()))
            .map_err(|_| panic!("Failed to set texture creator"))
            .unwrap();

        Self {
            texture_id: 0,
            canvas,
            atlas: Atlas::new(),
            shaders: HashMap::new(),
            active_shader: ("".into(), 0),
        }
    }

    fn surface(&self) -> &SurfaceRef {
        self.canvas.surface()
    }

    pub(crate) fn gl_present_surface(&self) {
        unsafe {
            let surface = self.surface();
            let pixel_buffer = surface.without_lock().unwrap();

            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint,
                surface.width() as GLsizei,
                surface.height() as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixel_buffer.as_ptr() as *const _,
            );
        }
    }

    pub(crate) fn draw_quad(&self, vao: u32, indices_len: usize) {
        let (_, program) = self.active_shader;

        unsafe {
            gl::UseProgram(program);
            gl::BindVertexArray(vao);
            gl::BindTexture(gl::TEXTURE_2D, self.texture_id);
            gl::DrawElements(
                gl::TRIANGLES,
                indices_len as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    pub fn load_font<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P, size: u16) {
        let label = label.to_string();

        let bytes = std::fs::read(path).expect("Failed to read font file");
        let font = fontdue::Font::from_bytes(bytes, Default::default())
            .expect("Failed to parse font file");

        self.atlas.fonts.insert(label, Font::new(font, size as f32));
    }

    pub fn load_shader<L: ToString, V: ToString, F: ToString>(
        &mut self,
        label: L,
        vertex_src: V,
        fragment_src: F,
    ) {
        let label = label.to_string();
        let vertex = vertex_src.to_string();
        let fragment = fragment_src.to_string();

        let program = unsafe { create_shader_program(&vertex, &fragment) };

        self.shaders.insert(label, program);
    }

    pub fn active_shader(&self) -> String {
        self.active_shader.0.to_string()
    }

    pub fn set_shader<L: ToString>(&mut self, label: L) {
        let label = label.to_string();

        if let Some(&program) = self.shaders.get(&label) {
            self.active_shader = (label.into(), program);
        } else {
            panic!("Shader not found: {}", label);
        }
    }

    pub(crate) fn clean_shaders(&self) {
        let programs = self.shaders.values();

        for program in programs.into_iter() {
            unsafe {
                gl::DeleteProgram(*program);
            }
        }
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

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub fn color(&self) -> Color {
        self.canvas.draw_color()
    }

    pub fn set_color(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
    }

    pub fn draw_line<P: Into<Vec2>, Q: Into<Vec2>>(&mut self, start: P, end: Q) {
        let start: Vec2 = start.into();
        let end: Vec2 = end.into();
        self.canvas
            .draw_fline(FPoint::from(start), FPoint::from(end))
            .unwrap();
    }

    pub fn draw_rect<P: Into<Vec2>, S: Into<Size>>(&mut self, pos: P, size: S) {
        let pos: Vec2 = pos.into();
        let size: Size = size.into();

        self.canvas.draw_frect(size.to_frect(pos)).unwrap();
    }

    pub fn fill_rect<P: Into<Vec2>, S: Into<Size>>(&mut self, pos: P, size: S) {
        let pos: Vec2 = pos.into();
        let size: Size = size.into();

        self.canvas.fill_frect(size.to_frect(pos)).unwrap();
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

    /// Draws an anti-aliased (smooth) arc
    /// The arc is drawn from start_angle to end_angle
    pub fn draw_aa_arc<P: Into<Vec2>, U: ToU32>(
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

        let kind = TextureKind::AAArc(radius, start_angle as i32, end_angle as i32);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(&kind) {
                texture
            } else {
                let texture = Texture::aa_arc(texture_creator(), radius, start_angle, end_angle);

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

    pub fn fill_arc<P: Into<Vec2>, U: ToU32>(
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

        let kind = TextureKind::FilledArc(radius, start_angle as i32, end_angle as i32);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(&kind) {
                texture
            } else {
                let texture = Texture::arc_fill(texture_creator(), radius, start_angle, end_angle);

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

    pub fn fill_aa_arc<P: Into<Vec2>, U: ToU32>(
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

        let kind = TextureKind::AAFilledArc(radius, start_angle as i32, end_angle as i32);

        let texture = {
            if let Some(texture) = self.atlas.get_texture(&kind) {
                texture
            } else {
                let texture =
                    Texture::aa_arc_fill(texture_creator(), radius, start_angle, end_angle);

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
            let dst = Size::from((query.width, query.height)).to_frect(pos);

            self.canvas.copy_f(texture, None, dst).unwrap();
        }
    }

    pub fn fill_text<T: ToString, P: Into<Vec2>>(&mut self, text: T, pos: P, color: Color) {
        let text = text.to_string();
        let pos: Vec2 = pos.into();
        let (r, g, b) = color.rgb();

        let glyphs = {
            let font = self.atlas.get_current_font();

            font.layout
                .append(&[font.inner.clone()], &TextStyle::new(&text, font.size, 0));

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

        for (ch, (x, y, width, height)) in glyphs {
            if ch.is_whitespace() {
                continue;
            }

            let size = Size::from((width, height));

            if let Some(texture) = self.atlas.get_glyph(ch) {
                let dst = size.to_frect((pos.x + x, pos.y + y).into());

                texture.set_color_mod(r, g, b);
                self.canvas.copy_f(texture, None, dst).unwrap();
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
