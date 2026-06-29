pub mod batcher;
pub mod handle;

use gpu::PipelineCache;

use crate::Color;
use crate::Vertex;
use crate::immediate::batcher::Batcher;

pub struct ImmediateRenderer {
    pub(crate) draw_color: Color,

    pub(crate) triangle_batcher: Batcher<Vertex>,
}

impl ImmediateRenderer {
    pub(crate) fn new() -> Self {
        Self {
            draw_color: Color::White,
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

        //rp.set_bind_group(0, camera_bg, &[]);

        self.triangle_batcher
            .present(rp, pipelines.get_pipeline(&desc));
    }
}
