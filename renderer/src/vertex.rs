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

/// Vertex type Specifically used for rendering
/// circles in immediate mode via `draw.cirlce()`
///
/// Uses a shader for cutting out pixels and make it
/// into a circle.
#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CircleVertex {
    pub position: math::Vector3<f32>, // 12 bytes
    pub color: math::Vector4<f32>,    // 16 bytes
    pub center: math::Vector2<f32>,   // 8 bytes
    pub radius: f32,                  // 4 bytes
}

impl CircleVertex {
    #[inline]
    pub fn new(
        position: math::Vector3<f32>,
        color: math::Vector4<f32>,
        center: math::Vector2<f32>,
        radius: f32,
    ) -> Self {
        Self {
            position,
            color,
            center,
            radius,
        }
    }
}
