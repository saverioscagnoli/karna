pub mod batcher;
pub mod imgui;

use std::sync::Arc;

use crate::Shape;
use crate::immediate::batcher::Batcher;
use crate::layouts::LayoutDesc;
use crate::layouts::{self};
use crate::vertex::ShapeVertex;

pub struct ImmediateRenderer {
    batcher: Batcher<ShapeVertex>,
    pipeline: Option<(wgpu::TextureFormat, Arc<wgpu::RenderPipeline>)>,
}

impl ImmediateRenderer {
    pub fn new() -> Self {
        Self {
            batcher: Batcher::new(),
            pipeline: None,
        }
    }

    pub fn present<'rp>(
        &'rp mut self,
        shapes: &Shape<ShapeVertex>,
        pass: &mut wgpu::RenderPass<'rp>,
        pipelines: &gpu::PipelineCache,
        format: wgpu::TextureFormat,
    ) {
        if shapes.is_empty() {
            return;
        }

        self.batcher.upload(&shapes.vertices, &shapes.indices);

        let stale = !matches!(&self.pipeline, Some((f, _)) if *f == format);

        if stale {
            let desc = gpu::PipelineDesc {
                shader: "immediate-2d",
                vertex_layout: ShapeVertex::desc(),
                blend: wgpu::BlendState::ALPHA_BLENDING,
                topology: wgpu::PrimitiveTopology::TriangleList,
                instance_layout: None,
                depth: gpu::DepthMode::Disabled,
                cull: None,
            };

            let pipeline = pipelines.get_or_create(desc, format, &layouts::immediate());
            self.pipeline = Some((format, pipeline));
        }

        let (_, pipeline) = self.pipeline.as_ref().unwrap();
        self.batcher.present(pass, Some(pipeline));
    }
}
