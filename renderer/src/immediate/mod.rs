mod batcher;

pub mod handle;
pub mod immediate_2d;

use immediate_2d as shd;
use math::Vector4;
use sokol::gfx as sg;

use crate::immediate::batcher::Batcher;
use crate::pipeline::PipelineCache;
use crate::pipeline::PipelineDesc;
use crate::vertex;
use crate::vertex::Vertex;

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

    /// Flushes both batchers. `vp` is the layer camera's view-projection,
    /// already packed into the generated uniform struct.
    pub(crate) fn present(&mut self, pipelines: &PipelineCache, vp: &shd::VsParams) {
        let point_desc = PipelineDesc {
            shader: "immediate-2d",
            topology: sg::PrimitiveType::Points,
            blend: true,
        };

        self.point_batcher
            .present(pipelines.get_pipeline(&point_desc), vp);

        let tri_desc = PipelineDesc {
            shader: "immediate-2d",
            topology: sg::PrimitiveType::Triangles,
            blend: true,
        };

        self.triangle_batcher
            .present(pipelines.get_pipeline(&tri_desc), vp);
    }
}
