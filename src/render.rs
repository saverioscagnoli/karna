use crate::math::{Size, Vector2};
use sdl2::{
    rect::{FPoint, FRect},
    render::{Canvas, Texture, TextureCreator},
    ttf::{Font, Sdl2TtfContext},
    video::{Window, WindowContext},
};
use std::{cell::OnceCell, collections::HashMap, fmt::Debug, path::Path};

static mut TTF_CTX: OnceCell<Sdl2TtfContext> = OnceCell::new();
static mut FONTS: OnceCell<HashMap<String, Font>> = OnceCell::new();
static mut FONT_TEXTURE_CACHE: OnceCell<HashMap<String, Texture>> = OnceCell::new();

pub(crate) fn init() {
    unsafe {
        TTF_CTX
            .set(sdl2::ttf::init().unwrap())
            .unwrap_or_else(|_| panic!("Failed to set TTF context"));

        FONTS
            .set(HashMap::new())
            .unwrap_or_else(|_| panic!("Failed to set fonts"));

        FONT_TEXTURE_CACHE
            .set(HashMap::new())
            .unwrap_or_else(|_| panic!("Failed to set font texture cache"));
    }
}

pub fn load_font(label: impl ToString, path: impl AsRef<Path>, size: u16) {
    let label = label.to_string();
    let path = path.as_ref();

    unsafe {
        let ctx = TTF_CTX.get().unwrap();
        let font = ctx.load_font(path, size).unwrap();
        FONTS.get_mut().unwrap().insert(label, font);
    }
}

fn get_font(label: impl ToString) -> &'static Font<'static, 'static> {
    let label = label.to_string();

    unsafe {
        let fonts = FONTS.get().unwrap();
        fonts.get(&label).unwrap()
    }
}

fn process_font_texture(
    text: impl ToString,
    color: Color,
    texture_creator: &TextureCreator<WindowContext>,
) -> &Texture {
    let text = text.to_string();
    let color = color.sdl();

    let key = format!("{}-{:?}", text, color);
    let key = key.as_str();

    unsafe {
        if let Some(texture) = FONT_TEXTURE_CACHE.get().unwrap().get(key) {
            return texture;
        }

        let font = get_font("default");
        let surface = font.render(&text).blended(color).unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();

        FONT_TEXTURE_CACHE
            .get_mut()
            .unwrap()
            .insert(key.to_string(), texture);

        FONT_TEXTURE_CACHE.get().unwrap().get(key).unwrap()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    Red,
    Blue,
    Green,
    White,
    Black,
    Cyan,
    Magenta,
    Yellow,
    Orange,
    Purple,
    Brown,
    Pink,
    Gray,
    Transparent,
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
    Hex(u32),
    HexAlpha(u32, u8),
}

impl Color {
    pub(crate) fn sdl(&self) -> sdl2::pixels::Color {
        match self {
            Color::Red => sdl2::pixels::Color::RED,
            Color::Blue => sdl2::pixels::Color::BLUE,
            Color::Green => sdl2::pixels::Color::GREEN,
            Color::White => sdl2::pixels::Color::WHITE,
            Color::Black => sdl2::pixels::Color::BLACK,
            Color::Cyan => sdl2::pixels::Color::CYAN,
            Color::Magenta => sdl2::pixels::Color::MAGENTA,
            Color::Yellow => sdl2::pixels::Color::YELLOW,
            Color::Orange => sdl2::pixels::Color::RGB(255, 165, 0),
            Color::Purple => sdl2::pixels::Color::RGB(128, 0, 128),
            Color::Brown => sdl2::pixels::Color::RGB(165, 42, 42),
            Color::Pink => sdl2::pixels::Color::RGB(255, 192, 203),
            Color::Gray => sdl2::pixels::Color::RGB(128, 128, 128),
            Color::Transparent => sdl2::pixels::Color::RGBA(0, 0, 0, 0),
            Color::RGB(r, g, b) => sdl2::pixels::Color::RGB(*r, *g, *b),
            Color::RGBA(r, g, b, a) => sdl2::pixels::Color::RGBA(*r, *g, *b, *a),
            Color::Hex(hex) => sdl2::pixels::Color::RGB(
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
            ),
            Color::HexAlpha(hex, a) => sdl2::pixels::Color::RGBA(
                ((hex >> 16) & 0xFF) as u8,
                ((hex >> 8) & 0xFF) as u8,
                (hex & 0xFF) as u8,
                *a,
            ),
        }
    }
}

pub struct Renderer {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    font: String,
}

impl Renderer {
    pub(crate) fn new(canvas: Canvas<Window>) -> Self {
        let texture_creator = canvas.texture_creator();

        Self {
            canvas,
            texture_creator,
            font: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn set_color(&mut self, color: Color) {
        self.canvas.set_draw_color(color.sdl());
    }

    pub fn set_font(&mut self, label: impl ToString) {
        self.font = label.to_string();
    }

    pub fn draw_rect(&mut self, pos: impl Into<Vector2>, size: impl Into<Size>) {
        let pos = pos.into();
        let size = size.into();

        self.canvas
            .draw_frect(FRect::new(pos.x, pos.y, size.width, size.height))
            .unwrap();
    }

    pub fn fill_rect(&mut self, pos: impl Into<Vector2>, size: impl Into<Size>) {
        let pos = pos.into();
        let size = size.into();

        self.canvas
            .fill_frect(FRect::new(pos.x, pos.y, size.width, size.height))
            .unwrap();
    }

    pub fn fill_text(&mut self, text: impl ToString, pos: impl Into<Vector2>, color: Color) {
        let text = text.to_string();
        let pos = pos.into();

        // First, fetch the texture without borrowing `self.canvas` mutably
        let texture = process_font_texture(text, color, &self.texture_creator);

        // Now, perform the mutable borrow to use `self.canvas`
        let query = texture.query();

        self.canvas
            .copy_f(
                texture, // Passing the immutable reference to the texture
                None,
                Some(FRect::new(
                    pos.x,
                    pos.y,
                    query.width as f32,
                    query.height as f32,
                )),
            )
            .unwrap();
    }

    pub fn fill_text_ex(
        &mut self,
        text: impl ToString,
        pos: impl Into<Vector2>,
        color: Color,
        scale: Option<f32>,
        angle: Option<f64>,
        center: Option<(f32, f32)>,
        flip_horizontal: bool,
        flip_vertical: bool,
    ) {
        let text = text.to_string();
        let pos = pos.into();

        let texture = process_font_texture(text, color, &self.texture_creator);

        let scale = scale.unwrap_or(1.0);
        let query = texture.query();

        self.canvas
            .copy_ex_f(
                texture,
                None,
                Some(FRect::new(
                    pos.x,
                    pos.y,
                    query.width as f32 * scale,
                    query.height as f32 * scale,
                )),
                angle.unwrap_or(0.0),
                center.map(|v| FPoint::new(v.0, v.1)),
                flip_horizontal,
                flip_vertical,
            )
            .unwrap();
    }
}
