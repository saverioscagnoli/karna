use macros::{Get, Set, With};
use math::Vector4;

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

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
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
