#![allow(non_upper_case_globals)]

use math::{Vec3, Vec4};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const Red: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const Green: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const Blue: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const White: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const Black: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const Yellow: Self = Self::rgb(1.0, 1.0, 0.0);
    pub const Magenta: Self = Self::rgb(1.0, 0.0, 1.0);
    pub const Cyan: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const Orange: Self = Self::rgb(1.0, 0.5, 0.0);
    pub const Purple: Self = Self::rgb(0.5, 0.0, 1.0);
    pub const Gray: Self = Self::rgb(0.5, 0.5, 0.5);
    pub const DarkGray: Self = Self::rgb(0.25, 0.25, 0.25);
    pub const LightGray: Self = Self::rgb(0.75, 0.75, 0.75);
    pub const Transparent: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::White
    }
}

impl From<Vec3> for Color {
    fn from(vec: Vec3) -> Self {
        Self {
            r: vec.x,
            g: vec.y,
            b: vec.z,
            a: 1.0,
        }
    }
}

impl From<Color> for Vec3 {
    fn from(col: Color) -> Self {
        [col.r, col.g, col.b].into()
    }
}

impl From<Vec4> for Color {
    fn from(vec: Vec4) -> Self {
        Self {
            r: vec.x,
            g: vec.y,
            b: vec.z,
            a: vec.w,
        }
    }
}

impl From<Color> for Vec4 {
    fn from(col: Color) -> Self {
        [col.r, col.g, col.b, col.a].into()
    }
}

impl From<Color> for wgpu::Color {
    fn from(col: Color) -> Self {
        Self {
            r: col.r as f64,
            g: col.g as f64,
            b: col.b as f64,
            a: col.a as f64,
        }
    }
}
