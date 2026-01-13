use macros::{Get, Set, With};
use math::{Vector4, rng};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[derive(Get, Set, With)]
pub struct Color {
    #[get(copied, name = "red")]
    #[set(name = "set_red")]
    #[with(name = "with_red")]
    pub r: f32,

    #[get(copied, name = "green")]
    #[set(name = "set_green")]
    #[with(name = "with_green")]
    pub g: f32,

    #[get(copied, name = "blue")]
    #[set(name = "set_blue")]
    #[with(name = "with_blue")]
    pub b: f32,

    #[get(copied, name = "alpha")]
    #[set(name = "set_alpha")]
    #[with(name = "with_alpha")]
    pub a: f32,
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

    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a Color from HSV (Hue, Saturation, Value) color space
    ///
    /// # Arguments
    /// * `hue` - Hue value in degrees (0.0 to 360.0)
    /// * `saturation` - Saturation (0.0 to 1.0)
    /// * `value` - Value/Brightness (0.0 to 1.0)
    pub fn hsv(hue: f32, saturation: f32, value: f32) -> Self {
        let saturation = saturation.clamp(0.0, 1.0);
        let value = value.clamp(0.0, 1.0);
        let hue = hue % 360.0;
        let hue = if hue < 0.0 { hue + 360.0 } else { hue };

        let c = value * saturation;
        let x = c * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
        let m = value - c;

        let (r, g, b) = if hue < 60.0 {
            (c, x, 0.0)
        } else if hue < 120.0 {
            (x, c, 0.0)
        } else if hue < 180.0 {
            (0.0, c, x)
        } else if hue < 240.0 {
            (0.0, x, c)
        } else if hue < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        Self::rgb(r + m, g + m, b + m)
    }

    /// Creates a Color from HSVA (HSV with Alpha)
    pub fn from_hsva(hue: f32, saturation: f32, value: f32, alpha: f32) -> Self {
        Self::hsv(hue, saturation, value).with_alpha(alpha)
    }

    #[inline]
    pub fn random() -> Self {
        let r = rng(0.0..=1.0);
        let g = rng(0.0..=1.0);
        let b = rng(0.0..=1.0);

        Self { r, g, b, a: 1.0 }
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::Black
    }
}

impl From<Vector4> for Color {
    fn from(value: Vector4) -> Self {
        Self::rgba(value.x, value.y, value.z, value.w)
    }
}

impl From<Color> for Vector4 {
    fn from(value: Color) -> Self {
        Vector4::new(value.r, value.g, value.b, value.a)
    }
}

impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Self {
        Self::rgba(value[0], value[1], value[2], value[3])
    }
}

impl From<Color> for [f32; 4] {
    fn from(value: Color) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}

impl From<wgpu::Color> for Color {
    fn from(value: wgpu::Color) -> Self {
        Self::rgba(
            value.r as f32,
            value.g as f32,
            value.b as f32,
            value.a as f32,
        )
    }
}

impl From<Color> for wgpu::Color {
    fn from(value: Color) -> Self {
        Self {
            r: value.r as f64,
            g: value.g as f64,
            b: value.b as f64,
            a: value.a as f64,
        }
    }
}
