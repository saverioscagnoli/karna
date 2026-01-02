mod line;
mod point;
mod triangle;

use crate::{
    Camera, Descriptor, Vertex,
    immediate::{line::LineBatcher, point::PointBatcher, triangle::TriangleBatcher},
    shader::Shader,
};
use assets::AssetManager;
use math::Vector4;
use std::sync::Arc;
use utils::{Label, label};

pub struct ImmediateRenderer {
    assets: Arc<AssetManager>,
    point_batcher: PointBatcher,
    line_batcher: LineBatcher,
    triangle_batcher: TriangleBatcher,
    zstep: f32,
}

impl ImmediateRenderer {
    const BASE_VERTEX_CAPACITY: usize = 1024;
    const BASE_INDEX_CAPACITY: usize = 1024;

    pub(crate) fn new(
        surface_format: wgpu::TextureFormat,
        camera: &Camera,
        assets: Arc<AssetManager>,
    ) -> Self {
        let shader = Shader::from_wgsl_file(
            include_str!("../../../shaders/immediate.wgsl"),
            Some("Immediate shader module"),
        );

        let point_pipeline = shader
            .pipeline_builder()
            .label("immediate line pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::PointList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc()],
            );

        let line_pipeline = shader
            .pipeline_builder()
            .label("immediate line pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::LineList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc()],
            );

        let line_stip_pipeline = shader
            .pipeline_builder()
            .label("immediate line pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::LineStrip)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc()],
            );

        let triangle_pipeline = shader
            .pipeline_builder()
            .label("immediate triangle pipeline")
            .vertex_entry("vs_main")
            .fragment_entry("fs_main")
            .topology(wgpu::PrimitiveTopology::TriangleList)
            .blend_state(Some(wgpu::BlendState::ALPHA_BLENDING))
            .build(
                surface_format,
                &[
                    camera.view_projection_bind_group_layout(),
                    assets.bind_group_layout(),
                ],
                &[Vertex::desc()],
            );

        let point_batcher = PointBatcher::new(point_pipeline);
        let line_batcher = LineBatcher::new(line_pipeline, line_stip_pipeline);
        let triangle_batcher = TriangleBatcher::new(triangle_pipeline);

        Self {
            assets,
            point_batcher,
            line_batcher,
            triangle_batcher,
            zstep: 0.0,
        }
    }

    #[inline]
    pub fn draw_point(&mut self, x: f32, y: f32, color: Vector4) {
        self.point_batcher.draw_point(x, y, self.zstep, color);
    }

    #[inline]
    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Vector4) {
        self.line_batcher
            .draw_line(x1, y1, x2, y2, self.zstep, color);
    }

    #[inline]
    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Vector4) {
        self.line_batcher.stroke_rect(x, y, self.zstep, w, h, color);
    }

    #[inline]
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Vector4) {
        self.triangle_batcher
            .fill_rect(x, y, self.zstep, w, h, color, &self.assets);
    }

    #[inline]
    pub fn draw_image(&mut self, label: Label, x: f32, y: f32, tint: Vector4) {
        self.triangle_batcher
            .draw_image(label, x, y, self.zstep, tint, &self.assets);
    }

    #[inline]
    pub fn draw_subimage(
        &mut self,
        label: Label,
        x: f32,
        y: f32,
        sx: f32,
        sy: f32,
        sw: f32,
        sh: f32,
        tint: Vector4,
    ) {
        self.triangle_batcher.draw_subimage(
            label,
            x,
            y,
            self.zstep,
            sx,
            sy,
            sw,
            sh,
            tint,
            &self.assets,
        );
    }

    #[inline]
    pub fn draw_text(&mut self, font_label: Label, text: &str, x: f32, y: f32, color: Vector4) {
        self.triangle_batcher
            .draw_text(font_label, text, x, y, self.zstep, color, &self.assets);
    }

    #[inline]
    pub fn debug_text(&mut self, text: &str, x: f32, y: f32, color: Vector4) {
        self.triangle_batcher.draw_text(
            label!("debug"),
            text,
            x,
            y,
            self.zstep,
            color,
            &self.assets,
        );
    }

    #[inline]
    pub(crate) fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.point_batcher.present(render_pass);
        self.line_batcher.present(render_pass);
        self.triangle_batcher.present(render_pass);

        self.zstep = 0.0;
    }
}
