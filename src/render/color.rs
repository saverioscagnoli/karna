#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Black,
    Transparent,
    RGB(u8, u8, u8),
    RGBA(u8, u8, u8, u8),
    Hex(u32),
}

impl Color {
    pub fn rgba(&self) -> (u8, u8, u8, u8) {
        match self {
            Color::White => (255, 255, 255, 255),
            Color::Red => (255, 0, 0, 255),
            Color::Green => (0, 255, 0, 255),
            Color::Blue => (0, 0, 255, 255),
            Color::Yellow => (255, 255, 0, 255),
            Color::Cyan => (0, 255, 255, 255),
            Color::Magenta => (255, 0, 255, 255),
            Color::Black => (0, 0, 0, 255),
            Color::Transparent => (0, 0, 0, 0),
            Color::RGB(r, g, b) => (*r, *g, *b, 255),
            Color::RGBA(r, g, b, a) => (*r, *g, *b, *a),
            Color::Hex(hex) => {
                let r = ((hex >> 16) & 0xFF) as u8;
                let g = ((hex >> 8) & 0xFF) as u8;
                let b = (hex & 0xFF) as u8;
                (r, g, b, 255)
            }
        }
    }

    pub fn rgb(&self) -> (u8, u8, u8) {
        let (r, g, b, _) = self.rgba();
        (r, g, b)
    }

    pub fn hex(&self) -> u32 {
        match self {
            Color::White => 0xFFFFFF,
            Color::Red => 0xFF0000,
            Color::Green => 0x00FF00,
            Color::Blue => 0x0000FF,
            Color::Yellow => 0xFFFF00,
            Color::Cyan => 0x00FFFF,
            Color::Magenta => 0xFF00FF,
            Color::Black => 0x000000,
            Color::Transparent => 0x000000,
            Color::RGB(r, g, b) => ((*r as u32) << 16) | ((*g as u32) << 8) | *b as u32,
            Color::RGBA(r, g, b, _) => ((*r as u32) << 16) | ((*g as u32) << 8) | *b as u32,
            Color::Hex(hex) => *hex,
        }
    }
}

impl From<Color> for sdl2::pixels::Color {
    fn from(color: Color) -> Self {
        let (r, g, b, a) = color.rgba();
        sdl2::pixels::Color::RGBA(r, g, b, a)
    }
}
