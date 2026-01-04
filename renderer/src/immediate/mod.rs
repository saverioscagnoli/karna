mod batcher;
mod handle;

use crate::{
    Camera, color::Color, immediate::batcher::Batcher, immediate_shader, shader::Shader,
    traits::LayoutDescriptor, vertex::Vertex,
};
use assets::AssetManager;
use macros::{Get, Set};

pub use handle::*;
use math::{Vector2, Vector4};

#[derive(Debug)]
#[derive(Get, Set)]
pub struct ImmediateRenderer {
    triangle_batcher: Batcher,

    pub(crate) draw_color: Color,
}

impl ImmediateRenderer {
    pub(crate) fn new(
        surface_format: wgpu::TextureFormat,
        camera: &Camera,
        assets: &AssetManager,
    ) -> Self {
        let triangle_pipeline = immediate_shader()
            .pipeline_builder()
            .label("Immediate Triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[camera.bgl(), assets.bind_group_layout()],
                &[Vertex::desc()],
            );

        let triangle_batcher = Batcher::new(triangle_pipeline);

        Self {
            draw_color: Color::Black,
            triangle_batcher,
        }
    }

    #[inline]
    pub fn fill_rect(&mut self, pos: Vector2, w: f32, h: f32, assets: &AssetManager) {
        let color: Vector4 = self.draw_color.into();

        // Get white pixel UV coords for solid color rendering
        let (uv_x, uv_y, uv_w, uv_h) = assets.get_white_uv_coords();
        let uv_center: Vector2 = [uv_x + uv_w * 0.5, uv_y + uv_h * 0.5].into();
        let base = self.triangle_batcher.vertices.len() as u32;

        let [x, y] = pos.into();

        self.triangle_batcher.vertices.extend_from_slice(&[
            Vertex {
                position: [x, y, 0.0].into(),
                color,
                uv: uv_center,
            },
            Vertex {
                position: [x + w, y, 0.0].into(),
                color,
                uv: uv_center,
            },
            Vertex {
                position: [x + w, y + h, 0.0].into(),
                color,
                uv: uv_center,
            },
            Vertex {
                position: [x, y + h, 0.0].into(),
                color,
                uv: uv_center,
            },
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
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.triangle_batcher.present(render_pass);
    }
}
