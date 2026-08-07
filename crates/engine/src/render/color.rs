use math::Vector4;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::White
    }
}

#[allow(non_upper_case_globals)]
impl Color {
    pub const Red: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const Green: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const Blue: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const White: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const Black: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const Yellow: Self = Self::rgb(1.0, 1.0, 0.0);
    pub const Cyan: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const Magenta: Self = Self::rgb(1.0, 0.0, 1.0);
    pub const Gray: Self = Self::rgb(0.5, 0.5, 0.5);
    pub const Orange: Self = Self::rgb(1.0, 0.65, 0.0);
    pub const Purple: Self = Self::rgb(0.5, 0.0, 0.5);
    pub const Brown: Self = Self::rgb(0.6, 0.3, 0.0);
    pub const Pink: Self = Self::rgb(1.0, 0.75, 0.8);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.0)
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
}

impl From<Vector4<f32>> for Color {
    fn from(v: Vector4<f32>) -> Self {
        Self {
            r: v.x,
            g: v.y,
            b: v.z,
            a: v.w,
        }
    }
}

impl Into<Vector4<f32>> for Color {
    fn into(self) -> Vector4<f32> {
        Vector4::new(self.r, self.g, self.b, self.a)
    }
}

impl From<[f32; 4]> for Color {
    fn from(v: [f32; 4]) -> Self {
        Self {
            r: v[0],
            g: v[1],
            b: v[2],
            a: v[3],
        }
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}
