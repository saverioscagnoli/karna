mod batcher;

use std::mem;

use math::Vector2;
use math::Vector3;
use math::Vector4;

use crate::Color;
use crate::immediate::batcher::Batcher;
use crate::immediate_shader;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateVertex {
    pub position: Vector3,
    pub color: Vector4,
}

impl ImmediateVertex {
    fn new(p: Vector3, color: Vector4) -> Self {
        Self { position: p, color }
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

pub struct ImmediateRenderer {
    draw_color: Color,
    triangle_batcher: Batcher<ImmediateVertex>,
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

        Self {
            draw_color: Color::Gray,
            triangle_batcher: Batcher::new(triangle_pipeline),
        }
    }

    #[inline]
    pub fn fill_rect(&mut self, pos: Vector2, w: f32, h: f32) {
        let color: Vector4 = self.draw_color.into();

        let base = self.triangle_batcher.vertices.len() as u32;

        self.triangle_batcher.vertices.extend_from_slice(&[
            ImmediateVertex::new(pos.extend(0.0), color),
            ImmediateVertex::new(Vector3::new(pos.x + w, pos.y, 0.0), color),
            ImmediateVertex::new(Vector3::new(pos.x + w, pos.y + h, 0.0), color),
            ImmediateVertex::new(Vector3::new(pos.x, pos.y + h, 0.0), color),
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
    pub fn present<'pass>(&'pass mut self, render_pass: &mut wgpu::RenderPass<'pass>) {
        self.triangle_batcher.present(render_pass);
    }
}
