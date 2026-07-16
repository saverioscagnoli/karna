pub mod batcher;
pub mod imgui;

use assets::AssetsRead;

use crate::Shape;
use crate::immediate::batcher::Batcher;
use crate::layouts::LayoutDesc;
use crate::layouts::{self};
use crate::vertex::CircleVertex;
use crate::vertex::Vertex;

pub struct ImmediateRenderer {
    point_batcher: Batcher<Vertex>,
    line_batcher: Batcher<Vertex>,
    triangle_batcher: Batcher<Vertex>,
    circle_batcher: Batcher<CircleVertex>,
}

impl ImmediateRenderer {
    pub fn new() -> Self {
        Self {
            point_batcher: Batcher::new(),
            line_batcher: Batcher::new(),
            triangle_batcher: Batcher::new(),
            circle_batcher: Batcher::new(),
        }
    }

    pub fn present<'rp>(
        &'rp mut self,
        points: &Shape<Vertex>,
        lines: &Shape<Vertex>,
        triangles: &Shape<Vertex>,
        cirlces: &Shape<CircleVertex>,
        pass: &mut wgpu::RenderPass<'rp>,
        pipelines: &gpu::PipelineCache,
        format: wgpu::TextureFormat,
        _assets: AssetsRead<'rp>,
    ) {
        if !points.is_empty() {
            self.point_batcher.upload(&points.vertices, &points.indices);

            let desc = gpu::PipelineDesc {
                shader: "immediate-2d",
                vertex_layout: Vertex::desc(),
                blend: wgpu::BlendState::ALPHA_BLENDING,
                topology: wgpu::PrimitiveTopology::PointList,
                instance_layout: None,
                depth: gpu::DepthMode::Disabled,
                cull: Some(wgpu::Face::Front),
            };

            let pipeline = pipelines.get_or_create(desc, format, &layouts::immediate());
            self.point_batcher.present(pass, Some(&pipeline));
        }

        if !lines.is_empty() {
            self.line_batcher.upload(&lines.vertices, &lines.indices);

            let desc = gpu::PipelineDesc {
                shader: "immediate-2d",
                vertex_layout: Vertex::desc(),
                blend: wgpu::BlendState::ALPHA_BLENDING,
                topology: wgpu::PrimitiveTopology::LineList,
                instance_layout: None,
                depth: gpu::DepthMode::Disabled,
                cull: Some(wgpu::Face::Front),
            };

            let pipeline = pipelines.get_or_create(desc, format, &layouts::immediate());

            self.line_batcher.present(pass, Some(&pipeline));
        }

        if !triangles.is_empty() {
            self.triangle_batcher
                .upload(&triangles.vertices, &triangles.indices);

            let desc = gpu::PipelineDesc {
                shader: "immediate-2d",
                vertex_layout: Vertex::desc(),
                blend: wgpu::BlendState::ALPHA_BLENDING,
                topology: wgpu::PrimitiveTopology::TriangleList,
                instance_layout: None,
                depth: gpu::DepthMode::Disabled,
                cull: Some(wgpu::Face::Front),
            };

            let pipeline = pipelines.get_or_create(desc, format, &layouts::immediate());

            self.triangle_batcher.present(pass, Some(&pipeline));
        }

        if !cirlces.is_empty() {
            self.circle_batcher
                .upload(&cirlces.vertices, &cirlces.indices);

            let desc = gpu::PipelineDesc {
                shader: "immediate-2d-circles",
                vertex_layout: CircleVertex::desc(),
                blend: wgpu::BlendState::ALPHA_BLENDING,
                topology: wgpu::PrimitiveTopology::TriangleList,
                instance_layout: None,
                depth: gpu::DepthMode::Disabled,
                cull: None,
            };

            let pipeline = pipelines.get_or_create(desc, format, &layouts::immediate());

            self.circle_batcher.present(pass, Some(&pipeline));
        }
    }
}
