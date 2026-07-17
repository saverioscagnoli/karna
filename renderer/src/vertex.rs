use math::Vector2;
use math::Vector3;
use math::Vector4;

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector4<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub fn new(p: Vector3<f32>, color: Vector4<f32>, uv: Vector2<f32>) -> Self {
        Self {
            position: p,
            color,
            uv,
        }
    }
}

pub const MODE_TEXTURED: u32 = 0;
pub const MODE_CIRCLE: u32 = 1;
pub const MODE_RRECT: u32 = 2;
pub const MODE_CAPSULE: u32 = 3;
pub const FLAG_NO_AA: u32 = 0x100;

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct ShapeVertex {
    pub position: math::Vector3<f32>, // offset  0, Float32x3
    pub color: math::Vector4<f32>,    // offset 12, Float32x4
    pub uv: math::Vector2<f32>,       // offset 28, Float32x2
    pub local: math::Vector2<f32>,    // offset 36, Float32x2
    pub params: math::Vector4<f32>,   // offset 44, Float32x4
    pub uv_rect: math::Vector4<f32>,  // offset 60, Float32x4 (min.xy, max.zw)
    pub mode: u32,                    // offset 76, Uint32 (mode | flags)
} // stride 80

impl ShapeVertex {
    pub fn new(
        position: math::Vector3<f32>,
        color: math::Vector4<f32>,
        uv: math::Vector2<f32>,
        local: math::Vector2<f32>,
        params: math::Vector4<f32>,
        uv_rect: math::Vector4<f32>,
        mode: u32,
    ) -> Self {
        Self {
            position,
            color,
            uv,
            local,
            params,
            uv_rect,
            mode,
        }
    }
}
