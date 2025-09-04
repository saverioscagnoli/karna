use math::Vec4;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::rgb(1.0, 0.0, 1.0);
    pub const GRAY: Self = Self::rgb(0.5, 0.5, 0.5);
    pub const DARK_GRAY: Self = Self::rgb(0.25, 0.25, 0.25);
    pub const LIGHT_GRAY: Self = Self::rgb(0.75, 0.75, 0.75);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Into<wgpu::Color> for Color {
    fn into(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }
}

impl Into<Color> for wgpu::Color {
    fn into(self) -> Color {
        Color {
            r: self.r as f32,
            g: self.g as f32,
            b: self.b as f32,
            a: self.a as f32,
        }
    }
}

impl Into<Vec4> for Color {
    fn into(self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, self.a)
    }
}

impl Into<Color> for Vec4 {
    fn into(self) -> Color {
        Color {
            r: self.x,
            g: self.y,
            b: self.z,
            a: self.w,
        }
    }
}
