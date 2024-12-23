use super::{
    atlas::{Atlas, TextureKind},
    font::Font,
};
use crate::{
    gl::{
        vertex_attrib_pointer, Ebo, OpenGLTexture, Program, Shader, ShaderKind, Uniform, Vao, Vbo,
    },
    math::{
        circles::{CircleFill, CircleOutline},
        Size, ToU32, Vec2,
    },
    traits::LoadSurface,
};
use fontdue::layout::TextStyle;
use hashbrown::HashMap;
use sdl2::{
    pixels::PixelFormatEnum,
    rect::{self, FPoint, FRect},
    render::{Canvas, Texture, TextureCreator},
    surface::{Surface, SurfaceContext},
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
    pub(crate) canvas: Canvas<Surface<'static>>,
    pub(crate) atlas: Atlas,

    vao: Vao,
    _vbo: Vbo,
    _ebo: Ebo,
    gl_tex: OpenGLTexture,

    default_program: Program,
    programs: HashMap<String, Rc<Program>>,

    current_program: Option<Rc<Program>>,
    programs_this_frame: Vec<Rc<Program>>,
}

impl Renderer {
    #[rustfmt::skip]
    const QUAD_VERTEX: [f32; 20] = [
            // Positions    // Texture Coords
            1.0,  1.0, 0.0,  1.0, 0.0, // top right
            1.0, -1.0, 0.0,  1.0, 1.0, // bottom right
           -1.0, -1.0, 0.0,  0.0, 1.0, // bottom left
           -1.0,  1.0, 0.0,  0.0, 0.0  // top left 
    ];

    #[rustfmt::skip]
    const QUAD_INDEX: [u32; 6] = [
        0, 1, 3, // first triangle
        1, 2, 3  // second triangle
    ];

    pub fn new(width: u32, height: u32) -> Self {
        let canvas = Surface::new(width, height, PixelFormatEnum::RGBA32)
            .unwrap()
            .into_canvas()
            .unwrap();

        TEXTURE_CREATOR
            .set(TC(canvas.texture_creator()))
            .map_err(|_| panic!("Failed to set texture creator"))
            .unwrap();

        let vs = Shader::from_str(include_str!("../../assets/vs.glsl"), ShaderKind::Vertex);
        let fs = Shader::from_str(include_str!("../../assets/fs.glsl"), ShaderKind::Fragment);

        let default_program = Program::new(vs, fs);

        let vao = Vao::new();
        let vbo = Vbo::new();
        let ebo = Ebo::new();

        vao.bind();

        vbo.bind();
        vbo.buffer_data(&Self::QUAD_VERTEX, gl::STATIC_DRAW);

        ebo.bind();
        ebo.buffer_data(&Self::QUAD_INDEX, gl::STATIC_DRAW);

        vertex_attrib_pointer(0, 3, 5, 0);
        vertex_attrib_pointer(1, 2, 5, 3);

        Vbo::unbind();
        Vao::unbind();

        let texture = OpenGLTexture::new();

        texture.bind();

        texture.parameteri(gl::TEXTURE_MIN_FILTER, gl::LINEAR);
        texture.parameteri(gl::TEXTURE_MAG_FILTER, gl::LINEAR);
        texture.parameteri(gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE);
        texture.parameteri(gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE);

        texture.image_2d(width, height, std::ptr::null());

        Self {
            canvas,
            atlas: Atlas::new(),
            vao,
            _vbo: vbo,
            _ebo: ebo,
            gl_tex: texture,
            default_program,
            current_program: None,
            programs: HashMap::new(),
            programs_this_frame: Vec::new(),
        }
    }

    pub fn load_font<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P, size: u16) {
        let label = label.to_string();

        let bytes = std::fs::read(path).expect("Failed to read font file");
        let font = fontdue::Font::from_bytes(bytes, Default::default())
            .expect("Failed to parse font file");

        self.atlas.fonts.insert(label, Font::new(font, size as f32));
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

    pub fn load_shader<L: ToString>(
        &mut self,
        label: L,
        vertex_shader: Shader,
        fragment_shader: Shader,
    ) {
        assert!(
            vertex_shader.kind == ShaderKind::Vertex,
            "Expected vertex shader, found fragment shader. Maybe did you swap the arguments?",
        );

        assert!(
            fragment_shader.kind == ShaderKind::Fragment,
            "Expected fragment shader, found vertex shader. Maybe did you swap the arguments?",
        );

        let program = Program::new(vertex_shader, fragment_shader);
        self.programs.insert(label.to_string(), program.into());
    }

    pub fn set_shader<L: ToString>(&mut self, label: L) {
        let label = label.to_string();

        if let Some(program) = self.programs.get(&label) {
            self.programs_this_frame.push(program.clone());
            self.current_program = Some(program.clone());
        }
    }

    pub fn reset_shader(&mut self) {
        self.current_program = None;
    }

    pub fn set_shader_uniform<L: ToString>(&self, uniform: L, value: Uniform) {
        if let Some(program) = self.current_program.as_ref() {
            let uniform = uniform.to_string();
            program.set_uniform(&uniform, value);
        }
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub(crate) fn present(&mut self) {
        let surf = self.canvas.surface();
        let (width, height) = surf.size();
        let pixels = surf.without_lock().unwrap();

        self.default_program.r#use();

        self.gl_tex.bind();
        self.gl_tex
            .sub_image_2d(0, 0, width, height, pixels.as_ptr());

        self.vao.bind();

        unsafe {
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
        }

        Vao::unbind();
        self.programs_this_frame.clear();
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

    pub fn fill_rects<P: Into<Vec2>, S: Into<Size>, I: IntoIterator<Item = (P, S)>>(
        &mut self,
        rects: I,
    ) {
        let rects: Vec<_> = rects
            .into_iter()
            .map(|(pos, size)| {
                let pos: Vec2 = pos.into();
                let size: Size = size.into();

                size.to_frect(pos)
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
