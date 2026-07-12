use std::mem;

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
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vector3<f32>>() as wgpu::BufferAddress
                        + mem::size_of::<Vector4<f32>>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[macro_export]
macro_rules! vertex {
    // position, color as [r,g,b,a], uv as [u,v]
    (
        [$px:expr, $py:expr, $pz:expr],
        [$cr:expr, $cg:expr, $cb:expr, $ca:expr],
        [$u:expr, $v:expr]
    ) => {
        Vertex::new(
            Vector3::new($px as f32, $py as f32, $pz as f32),
            Vector4::new($cr as f32, $cg as f32, $cb as f32, $ca as f32),
            Vector2::new($u as f32, $v as f32),
        )
    };

    // position, color as [r,g,b,a], uv as Vector2 expr
    (
        [$px:expr, $py:expr, $pz:expr],
        [$cr:expr, $cg:expr, $cb:expr, $ca:expr],
        $uv:expr
    ) => {
        Vertex::new(
            Vector3::new($px as f32, $py as f32, $pz as f32),
            Vector4::new($cr as f32, $cg as f32, $cb as f32, $ca as f32),
            $uv,
        )
    };

    // position, color as expr (e.g. existing Vector4), uv as [u,v]
    (
        [$px:expr, $py:expr, $pz:expr],
        $color:expr,
        [$u:expr, $v:expr]
    ) => {
        Vertex::new(
            Vector3::new($px as f32, $py as f32, $pz as f32),
            $color,
            Vector2::new($u as f32, $v as f32),
        )
    };

    // position, color as expr, uv as Vector2 expr
    (
        [$px:expr, $py:expr, $pz:expr],
        $color:expr,
        $uv:expr
    ) => {
        Vertex::new(
            Vector3::new($px as f32, $py as f32, $pz as f32),
            $color,
            $uv,
        )
    };
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

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position: vec3<f32>
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color: vec4<f32>
                wgpu::VertexAttribute {
                    offset: mem::size_of::<math::Vector3<f32>>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // center: vec2<f32>
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<math::Vector3<f32>>()
                        + mem::size_of::<math::Vector4<f32>>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // radius: f32
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<math::Vector3<f32>>()
                        + mem::size_of::<math::Vector4<f32>>()
                        + mem::size_of::<math::Vector2<f32>>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}
