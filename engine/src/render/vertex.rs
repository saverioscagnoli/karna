use crate::render::LayoutDesc;

#[repr(C)]
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Vertex {
    pub position: math::Vector3<f32>,
    pub color: math::Vector4<f32>,
    pub uv: math::Vector2<f32>,
}

impl LayoutDesc for Vertex {
    const STEP_MODE: gpu::VertexStepMode = gpu::VertexStepMode::Vertex;
    const ATTRIBUTES: &'static [gpu::VertexAttribute] = &gpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
        2 => Float32x2
    ];
}
