#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateVertex {
    pub position: math::Vector3<f32>,
    pub color: math::Vector4<f32>,
    pub uv: math::Vector2<f32>,
    pub page: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MeshVertex {
    pub position: math::Vector3<f32>,
    pub normal: math::Vector3<f32>,
    pub uv: math::Vector2<f32>,
    pub color: math::Vector4<f32>,
}

impl MeshVertex {
    pub fn new(
        position: math::Vector3<f32>,
        normal: math::Vector3<f32>,
        uv: math::Vector2<f32>,
    ) -> Self {
        Self {
            position,
            normal,
            uv,
            color: math::Vector4::one(),
        }
    }

    pub fn with_color(mut self, color: math::Vector4<f32>) -> Self {
        self.color = color;
        self
    }
}
