mod batcher;
mod draw_handle;

use std::mem;

pub use draw_handle::Draw;
use macros::Get;
use macros::Set;
use math::Vector2;
use math::Vector3;
use math::Vector4;

use crate::Color;
use crate::immediate::batcher::Batcher;
use crate::immediate_circle_shader;
use crate::immediate_shader;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateVertex {
    pub position: Vector3,
    pub color: Vector4,
    pub uv: Vector2,
}

impl ImmediateVertex {
    fn new(x: f32, y: f32, z: f32, color: Vector4, u: f32, v: f32) -> Self {
        Self {
            position: Vector3::new(x, y, z),
            color,
            uv: Vector2::new(u, v),
        }
    }

    fn desc() -> wgpu::VertexBufferLayout<'static> {
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
                    offset: mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<Vector3>() as wgpu::BufferAddress
                        + mem::size_of::<Vector4>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
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
pub struct ImmediateCircleVertex {
    pub position: Vector3, // 12 bytes
    pub color: Vector4,    // 16 bytes
    pub center: Vector2,   // 8 bytes
    pub radius: f32,       // 4 bytes
}

impl ImmediateCircleVertex {
    #[inline]
    pub fn new(position: Vector3, color: Vector4, center: Vector2, radius: f32) -> Self {
        Self {
            position,
            color,
            center,
            radius,
        }
    }

    fn desc() -> wgpu::VertexBufferLayout<'static> {
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
                    offset: mem::size_of::<Vector3>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // center: vec2<f32>
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vector3>() + mem::size_of::<Vector4>())
                        as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // radius: f32
                wgpu::VertexAttribute {
                    offset: (mem::size_of::<Vector3>()
                        + mem::size_of::<Vector4>()
                        + mem::size_of::<Vector2>())
                        as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

#[derive(Get, Set)]
pub struct ImmediateRenderer {
    #[get]
    #[set(into)]
    draw_color: Vector4,
    triangle_batcher: Batcher<ImmediateVertex>,
    circle_batcher: Batcher<ImmediateCircleVertex>,
}

impl ImmediateRenderer {
    pub fn new(surface_format: wgpu::TextureFormat, camera_bgl: &wgpu::BindGroupLayout) -> Self {
        let triangle_pipeline = immediate_shader()
            .pipeline_builder()
            .label("Immediate Triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(surface_format, &[camera_bgl], &[ImmediateVertex::desc()]);

        let circle_pipeline = immediate_circle_shader()
            .pipeline_builder()
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera_bgl],
                &[ImmediateCircleVertex::desc()],
            );

        Self {
            draw_color: Color::White.into(),
            triangle_batcher: Batcher::new(triangle_pipeline),
            circle_batcher: Batcher::new(circle_pipeline),
        }
    }

    #[inline]
    pub fn push_quad(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let base = self.triangle_batcher.vertices.len() as u32;

        self.triangle_batcher.vertices.extend_from_slice(&[
            ImmediateVertex::new(x, y, 0.0, self.draw_color, 0.0, 0.0),
            ImmediateVertex::new(x + w, y, 0.0, self.draw_color, 1.0, 0.0),
            ImmediateVertex::new(x + w, y + h, 0.0, self.draw_color, 1.0, 1.0),
            ImmediateVertex::new(x, y + h, 0.0, self.draw_color, 0.0, 1.0),
        ]);

        self.triangle_batcher.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    #[inline]
    pub fn push_circle(&mut self, x: f32, y: f32, radius: f32) {
        let base = self.circle_batcher.vertices.len() as u32;
        let center = Vector2::new(x, y);

        self.circle_batcher.vertices.extend_from_slice(&[
            ImmediateCircleVertex::new(
                Vector3::new(x - radius, y - radius, 0.0),
                self.draw_color,
                center,
                radius,
            ),
            ImmediateCircleVertex::new(
                Vector3::new(x + radius, y - radius, 0.0),
                self.draw_color,
                center,
                radius,
            ),
            ImmediateCircleVertex::new(
                Vector3::new(x + radius, y + radius, 0.0),
                self.draw_color,
                center,
                radius,
            ),
            ImmediateCircleVertex::new(
                Vector3::new(x - radius, y + radius, 0.0),
                self.draw_color,
                center,
                radius,
            ),
        ]);

        self.circle_batcher.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base,
            base + 2,
            base + 3,
        ]);
    }

    #[inline]
    pub fn present<'pass>(&'pass mut self, render_pass: &mut wgpu::RenderPass<'pass>) {
        self.triangle_batcher.present(render_pass);
    }
}
