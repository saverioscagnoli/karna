#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateVertex {
    pub position: math::Vector3<f32>,
    pub color: math::Vector4<f32>,
    pub uv: math::Vector2<f32>,
    pub page: f32,
}
