pub mod batcher;
pub mod handle;
pub mod imgui;

use assets::AssetServerGuard;
use gpu::PipelineCache;
use math::Vector2;
use math::Vector3;
use math::Vector4;

use crate::Vertex;
use crate::immediate::batcher::Batcher;
use crate::vertex;
use crate::vertex::CircleVertex;

struct RenderState {}

pub struct ImmediateRenderer {
    pub point_batcher: Batcher<Vertex>,
    pub line_batcher: Batcher<Vertex>,
    pub triangle_batcher: Batcher<Vertex>,
    pub cirlce_batcher: Batcher<CircleVertex>,
}

impl ImmediateRenderer {
    pub(crate) fn new() -> Self {
        Self {
            point_batcher: Batcher::new(),
            line_batcher: Batcher::new(),
            triangle_batcher: Batcher::new(),
            cirlce_batcher: Batcher::new(),
        }
    }

    #[inline]
    fn push<V: Copy>(batcher: &mut Batcher<V>, vertices: &[V], pattern: &[u32]) {
        let base = batcher.vertex_count();

        batcher.vertices.extend_from_slice(vertices);
        batcher.indices.extend(pattern.iter().map(|i| base + i));
    }

    pub fn push_point<'a>(
        &mut self,
        x: f32,
        y: f32,
        color: Vector4<f32>,
        assets: &AssetServerGuard<'a>,
    ) {
        let white = assets.white_handle();
        let uv = assets.get_image(white).uv.xy();
        let v = vertex!([x, y, 0.0], color, uv);

        Self::push(&mut self.point_batcher, &[v], &[0]);
    }

    pub fn push_line<'a>(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: Vector4<f32>,
        assets: &AssetServerGuard<'a>,
    ) {
        let white = assets.white_handle();
        let uv = assets.get_image(white).uv.xy();
        let v = [
            vertex!([x1, y1, 0.0], color, uv),
            vertex!([x2, y2, 0.0], color, uv),
        ];

        Self::push(&mut self.line_batcher, &v, &[0, 1]);
    }

    pub fn push_untextured_quad<'a>(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: Vector4<f32>,
        assets: &AssetServerGuard<'a>,
    ) {
        let white = assets.white_handle();
        let uv = assets.get_image(white).uv;
        let uv_min = uv.xy();
        let uv_max = Vector2::new(uv.x + uv.z, uv.y + uv.w);

        let v = [
            vertex!([x, y, 0.0], color, [uv_min.x, uv_min.y]),
            vertex!([x + w, y, 0.0], color, [uv_max.x, uv_min.y]),
            vertex!([x, y + h, 0.0], color, [uv_min.x, uv_max.y]),
            vertex!([x + w, y + h, 0.0], color, [uv_max.x, uv_max.y]),
        ];

        Self::push(&mut self.triangle_batcher, &v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn push_textured_quad<'a>(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: math::Vector4<f32>,
        uv: math::Vector4<f32>,
    ) {
        let uv_min = uv.xy();
        let uv_max = Vector2::new(uv.x + uv.z, uv.y + uv.w); // (u0+uw, v0+vh)

        let v = [
            vertex!([x, y, 0.0], color, [uv_min.x, uv_min.y]), // top-left
            vertex!([x + w, y, 0.0], color, [uv_max.x, uv_min.y]), // top-right
            vertex!([x, y + h, 0.0], color, [uv_min.x, uv_max.y]), // bottom-left
            vertex!([x + w, y + h, 0.0], color, [uv_max.x, uv_max.y]), // bottom-right
        ];

        Self::push(&mut self.triangle_batcher, &v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn push_cirlce(&mut self, r: f32, x: f32, y: f32, color: math::Vector4<f32>) {
        let center = Vector2::new(x, y);

        let v = [
            CircleVertex::new(Vector3::new(x - r, y - r, 0.0), color, center, r), // top-left
            CircleVertex::new(Vector3::new(x + r, y - r, 0.0), color, center, r), // top-right
            CircleVertex::new(Vector3::new(x - r, y + r, 0.0), color, center, r), // bottom-left
            CircleVertex::new(Vector3::new(x + r, y + r, 0.0), color, center, r), // bottom-right
        ];

        Self::push(&mut self.cirlce_batcher, &v, &[0, 1, 2, 2, 1, 3]);
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
            topology: wgpu::PrimitiveTopology::LineList,
        };

        self.line_batcher.present(rp, pipelines.get_pipeline(&desc));

        let desc = gpu::PipelineDesc {
            shader: "immediate-2d",
            vertex_layout: Vertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
        };

        self.triangle_batcher
            .present(rp, pipelines.get_pipeline(&desc));

        let desc = gpu::PipelineDesc {
            shader: "immediate-2d-circles",
            vertex_layout: CircleVertex::desc(),
            blend: wgpu::BlendState::ALPHA_BLENDING,
            topology: wgpu::PrimitiveTopology::TriangleList,
        };

        self.cirlce_batcher
            .present(rp, pipelines.get_pipeline(&desc));
    }
}
