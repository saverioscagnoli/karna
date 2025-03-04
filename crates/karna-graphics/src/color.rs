#![allow(non_snake_case)]

use karna_math::vector::Vec4;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self::RGB(1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::RGB(0.0, 0.0, 0.0);
    pub const RED: Self = Self::RGB(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::RGB(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::RGB(0.0, 0.0, 1.0);
    pub const YELLOW: Self = Self::RGB(1.0, 1.0, 0.0);
    pub const CYAN: Self = Self::RGB(0.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::RGB(1.0, 0.0, 1.0);

    pub const fn RGB(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn RGBA(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn hex(hex: u32) -> Self {
        let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
        let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
        let b = (hex & 0xFF) as f32 / 255.0;

        Self::RGB(r, g, b)
    }
}

/// Color -> [r, g, b, a]
impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Color -> Vec4
impl Into<Vec4> for Color {
    fn into(self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, self.a)
    }
}
