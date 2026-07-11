mod batcher;

use gpu::Vertex;

use crate::Layouts;
use crate::Shape;
use crate::immediate::batcher::Batcher;

pub struct ImmediateRenderer {
    triangle_batcher: Batcher<Vertex>,
}

impl ImmediateRenderer {
    pub fn new() -> Self {
        Self {
            triangle_batcher: Batcher::new(),
        }
    }

    pub fn present<'rp>(
        &'rp mut self,
        triangles: &Shape<Vertex>,
        pass: &mut wgpu::RenderPass<'rp>,
        pipelines: &gpu::PipelineCache,
        layouts: &Layouts,
        format: wgpu::TextureFormat,
    ) {
        if triangles.is_empty() {
            return;
        }

        // Batcher now just owns GPU buffers; feed it CPU data.
        self.triangle_batcher
            .upload(&triangles.vertices, &triangles.indices);

        let desc = gpu::PipelineDesc {
            shader: "immediate-2d",
            vertex_layout: Vertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
        };

        let pipeline = pipelines.get_or_create(desc, format, &layouts.as_array());
        self.triangle_batcher.present(pass, &pipeline);
    }
}
