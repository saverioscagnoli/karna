pub mod batcher;
pub mod handle;

use gpu::PipelineCache;

use crate::Vertex;
use crate::immediate::batcher::Batcher;

pub struct ImmediateRenderer {
    pub(crate) triangle_batcher: Batcher<Vertex>,
}

impl ImmediateRenderer {
    pub(crate) fn new() -> Self {
        Self {
            triangle_batcher: Batcher::new(),
        }
    }

    pub fn present<'a>(&'a mut self, rp: &mut wgpu::RenderPass<'a>, pipelines: &PipelineCache) {
        let desc = gpu::PipelineDesc {
            shader: "immediate-2d",
            vertex_layout: Vertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
        };

        self.triangle_batcher
            .present(rp, pipelines.get_pipeline(&desc));
    }
}
