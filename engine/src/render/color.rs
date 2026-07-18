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

    pub fn to_linear_array(&self) -> [f32; 4] {
        [
            srgb_to_linear(self.r),
            srgb_to_linear(self.g),
            srgb_to_linear(self.b),
            srgb_to_linear(self.a),
        ]
    }
}

/// sRGB transfer function decode: one sRGB-encoded channel -> linear.
///
/// User-facing colors are sRGB (what you'd pick in a color picker); the
/// engine renders in linear space to an sRGB target, so sRGB inputs are
/// linearized at their entry points. Shader-side inputs are converted in
/// the shaders; CPU-side inputs (the clear color) use this.
pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

impl From<wgpu::Color> for Color {
    fn from(c: wgpu::Color) -> Self {
        Self {
            r: c.r as f32,
            g: c.g as f32,
            b: c.b as f32,
            a: c.a as f32,
        }
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
