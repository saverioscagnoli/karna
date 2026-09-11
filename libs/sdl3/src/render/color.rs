use sdl3_sys::SDL_FColor;

pub struct Color(SDL_FColor);

impl Color {
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self(SDL_FColor { r, g, b, a: 1.0 })
    }

    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(SDL_FColor { r, g, b, a })
    }

    pub const fn raw(&self) -> SDL_FColor {
        self.0
    }

    pub const fn hex(v: u32) -> Self {
        Self::rgba(
            ((v >> 16) & 0xFF) as f32 / 255.0,
            ((v >> 8) & 0xFF) as f32 / 255.0,
            (v & 0xFF) as f32 / 255.0,
            1.0,
        )
    }

    pub const fn hex_a(v: u32) -> Self {
        Self::rgba(
            ((v >> 24) & 0xFF) as f32 / 255.0,
            ((v >> 16) & 0xFF) as f32 / 255.0,
            ((v >> 8) & 0xFF) as f32 / 255.0,
            (v & 0xFF) as f32 / 255.0,
        )
    }

    pub fn try_hex(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);

        match s.len() {
            6 => u32::from_str_radix(s, 16).ok().map(Self::hex),
            8 => u32::from_str_radix(s, 16).ok().map(Self::hex_a),
            3 => {
                // expand #abc -> #aabbcc
                let mut v = 0u32;
                for c in s.chars() {
                    let d = c.to_digit(16)?;
                    v = (v << 8) | (d << 4) | d;
                }
                Some(Self::hex(v))
            }
            _ => None,
        }
    }

    pub const fn tuple(&self) -> (f32, f32, f32, f32) {
        (self.0.r, self.0.g, self.0.b, self.0.a)
    }

    pub const fn array(&self) -> [f32; 4] {
        [self.0.r, self.0.g, self.0.b, self.0.a]
    }
}

impl Color {
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::rgb(1.0, 0.0, 1.0);
    pub const GRAY: Self = Self::rgb(0.5, 0.5, 0.5);
    pub const ORANGE: Self = Self::rgb(1.0, 0.65, 0.0);
    pub const PURPLE: Self = Self::rgb(0.5, 0.0, 0.5);
    pub const BROWN: Self = Self::rgb(0.6, 0.3, 0.0);
    pub const PINK: Self = Self::rgb(1.0, 0.75, 0.8);
}

impl From<math::Vector4<f32>> for Color {
    fn from(v: math::Vector4<f32>) -> Self {
        Self(SDL_FColor {
            r: v.x,
            g: v.y,
            b: v.z,
            a: v.w,
        })
    }
}

impl Into<math::Vector4<f32>> for Color {
    fn into(self) -> math::Vector4<f32> {
        math::vec4!(self.0.r, self.0.g, self.0.b, self.0.a)
    }
}
