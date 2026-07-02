use math::Vector2;
use math::Vector3;
use math::Vector4;
use sokol::gfx as sg;

use crate::immediate::immediate_2d as shd;

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

    pub fn with_position(mut self, p: Vector3<f32>) -> Self {
        self.position = p;
        self
    }

    pub fn set_position(&mut self, p: Vector3<f32>) {
        self.position = p;
    }

    pub fn with_color(mut self, c: Vector4<f32>) -> Self {
        self.color = c;
        self
    }

    pub fn set_color(&mut self, c: Vector4<f32>) {
        self.color = c;
    }

    pub fn with_uv(mut self, uv: Vector2<f32>) -> Self {
        self.uv = uv;
        self
    }

    pub fn set_uv(&mut self, uv: Vector2<f32>) {
        self.uv = uv;
    }
}

impl Vertex {
    /// Vertex layout state for sokol pipelines. `shd` is the generated
    /// shader module (for ATTR_* slot constants).

    pub(crate) fn layout() -> sg::VertexLayoutState {
        let mut layout = sg::VertexLayoutState::new();

        // Tell sokol the real stride of the whole Vertex struct.
        layout.buffers[0].stride = std::mem::size_of::<Vertex>() as i32; // 36

        // Location 0: position (float3) at offset 0
        layout.attrs[shd::ATTR_SHADER_POSITION as usize] = sg::VertexAttrState {
            format: sg::VertexFormat::Float3,
            offset: 0,
            buffer_index: 0,
        };

        // Location 1: color (float4) at offset 12
        layout.attrs[shd::ATTR_SHADER_COLOR as usize] = sg::VertexAttrState {
            format: sg::VertexFormat::Float4,
            offset: std::mem::offset_of!(Vertex, color) as i32, // 12
            buffer_index: 0,
        };

        layout
    }
}

#[macro_export]
macro_rules! vertex {
    (
        [$px:expr, $py:expr, $pz:expr],
        [$cr:expr, $cg:expr, $cb:expr, $ca:expr],
        [$u:expr, $v:expr]
    ) => {
        Vertex::new(
            ::math::Vector3::new($px as f32, $py as f32, $pz as f32),
            ::math::Vector4::new($cr as f32, $cg as f32, $cb as f32, $ca as f32),
            ::math::Vector2::new($u as f32, $v as f32),
        )
    };
    (
        [$px:expr, $py:expr, $pz:expr],
        $color:expr,
        [$u:expr, $v:expr]
    ) => {
        Vertex::new(
            ::math::Vector3::new($px as f32, $py as f32, $pz as f32),
            $color,
            ::math::Vector2::new($u as f32, $v as f32),
        )
    };
}
