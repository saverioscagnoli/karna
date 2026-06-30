pub mod batcher;
pub mod handle;

use gpu::PipelineCache;
use math::Vector2;
use math::Vector3;
use math::Vector4;

use crate::Vertex;
use crate::immediate::batcher::Batcher;
use crate::vertex;

pub struct ImmediateRenderer {
    pub(crate) point_batcher: Batcher<Vertex>,
    pub(crate) triangle_batcher: Batcher<Vertex>,
}

impl ImmediateRenderer {
    pub(crate) fn new() -> Self {
        Self {
            point_batcher: Batcher::new(),
            triangle_batcher: Batcher::new(),
        }
    }

    pub fn push_point(&mut self, x: f32, y: f32, color: Vector4<f32>) {
        let base = self.point_batcher.vertex_count();

        self.point_batcher
            .vertices
            .push(vertex!([x, y, 0.0], color, [0.0, 0.0]));

        self.point_batcher.indices.push(base);
    }

    pub fn push_quad(&mut self, x: f32, y: f32, w: f32, h: f32, color: Vector4<f32>) {
        let base = self.triangle_batcher.vertex_count();

        let vertices: &[Vertex] = &[
            vertex!([x, y, 0.0], color, [0.0, 0.0]),
            vertex!([x + w, y, 0.0], color, [1.0, 0.0]),
            vertex!([x, y + h, 0.0], color, [0.0, 1.0]),
            vertex!([x + w, y + h, 0.0], color, [1.0, 1.0]),
        ];

        self.triangle_batcher.vertices.extend_from_slice(vertices);
        self.triangle_batcher.indices.extend_from_slice(&[
            base,
            base + 1,
            base + 2,
            base + 2,
            base + 1,
            base + 3,
        ]);
    }

    pub fn present<'a>(&'a mut self, rp: &mut wgpu::RenderPass<'a>, pipelines: &PipelineCache) {
        let desc = gpu::PipelineDesc {
            shader: "immediate-2d",
            vertex_layout: Vertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::PointList,
        };

        self.point_batcher
            .present(rp, pipelines.get_pipeline(&desc));

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
