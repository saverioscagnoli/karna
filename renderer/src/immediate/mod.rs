pub mod batcher;
pub mod handle;
pub mod imgui;

use assets::AssetServerView;
use assets::ReadOnly;
use gpu::CircleVertex;
use gpu::PipelineCache;
use gpu::Vertex;
use logging::warn;
use math::Matrix4;
use math::Vector2;
use math::Vector3;
use math::Vector4;

use crate::Color;
use crate::immediate::batcher::Batcher;

#[derive(Debug, Clone, Copy)]
pub struct RenderState {
    draw_color: Vector4<f32>,
    transform: Matrix4<f32>,
    depth: f32,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            draw_color: Color::White.into(),
            transform: Matrix4::identity(),
            depth: 0.0,
        }
    }
}

pub struct ImmediateRenderer {
    pub current_state: RenderState,
    pub state_stack: Vec<RenderState>,
    pub point_batcher: Batcher<Vertex>,
    pub line_batcher: Batcher<Vertex>,
    pub triangle_batcher: Batcher<Vertex>,
    pub cirlce_batcher: Batcher<CircleVertex>,
}

impl ImmediateRenderer {
    pub fn new() -> Self {
        Self {
            current_state: RenderState::default(),
            state_stack: Vec::new(),
            point_batcher: Batcher::new(),
            line_batcher: Batcher::new(),
            triangle_batcher: Batcher::new(),
            cirlce_batcher: Batcher::new(),
        }
    }

    #[inline]
    fn push_vertices<V: Copy>(batcher: &mut Batcher<V>, vertices: &[V], pattern: &[u32]) {
        let base = batcher.vertex_count();

        batcher.vertices.extend_from_slice(vertices);
        batcher.indices.extend(pattern.iter().map(|i| base + i));
    }

    pub fn push_state(&mut self) {
        self.state_stack.push(self.current_state);
    }

    pub fn pop_state(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.current_state = state;
        } else {
            warn!("Immediate renderer: popped a render state without pushing first");
        }
    }

    pub fn draw_color(&self) -> Vector4<f32> {
        self.current_state.draw_color
    }

    pub fn set_draw_color(&mut self, c: Vector4<f32>) {
        self.current_state.draw_color = c;
    }

    pub fn depth(&self) -> f32 {
        self.current_state.depth
    }

    pub fn set_depth(&mut self, d: f32) {
        self.current_state.depth = d;
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.current_state.transform = self
            .current_state
            .transform
            .matmul(&Matrix4::from_translation(Vector3::new(x, y, 0.0)))
    }

    pub fn rotate(&mut self, angle_rad: f32) {
        self.current_state.transform = self
            .current_state
            .transform
            .matmul(&Matrix4::from_rotation_z(angle_rad))
    }

    pub fn scale(&mut self, x: f32, y: f32) {
        self.current_state.transform = self
            .current_state
            .transform
            .matmul(&Matrix4::from_scale(Vector3::new(x, y, 0.0)))
    }

    pub fn tp(&self, x: f32, y: f32) -> Vector3<f32> {
        let pos = Vector4::new(x, y, 0.0, 1.0);
        let t = self.current_state.transform.mul_vec(&pos);

        Vector3::new(t.x, t.y, self.current_state.depth)
    }

    fn vertex(&self, x: f32, y: f32, uv: Vector2<f32>) -> Vertex {
        let p = self.tp(x, y);
        Vertex::new(p, self.current_state.draw_color, uv)
    }

    fn circle_vertex(&self, x: f32, y: f32, center: Vector2<f32>, r: f32) -> CircleVertex {
        let p = self.tp(x, y);
        let c = self.tp(center.x, center.y);
        CircleVertex::new(p, self.current_state.draw_color, Vector2::new(c.x, c.y), r)
    }

    pub fn push_point<'a>(&mut self, x: f32, y: f32, assets: &AssetServerView<'a, ReadOnly>) {
        let white = assets.white_handle();
        let uv = assets.get_image(white).uv.xy();
        let v = self.vertex(x, y, uv);

        Self::push_vertices(&mut self.point_batcher, &[v], &[0]);
    }

    pub fn push_line<'a>(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        assets: &AssetServerView<'a, ReadOnly>,
    ) {
        let white = assets.white_handle();
        let uv = assets.get_image(white).uv.xy();
        let v = [self.vertex(x1, y1, uv), self.vertex(x2, y2, uv)];

        Self::push_vertices(&mut self.line_batcher, &v, &[0, 1]);
    }

    pub fn push_untextured_quad<'a>(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        assets: &AssetServerView<'a, ReadOnly>,
    ) {
        let white = assets.white_handle();
        let uv = assets.get_image(white).uv;
        let uv_min = uv.xy();
        let uv_max = Vector2::new(uv.x + uv.z, uv.y + uv.w);

        let v = [
            self.vertex(x, y, uv_min),
            self.vertex(x + w, y, Vector2::new(uv_max.x, uv_min.y)),
            self.vertex(x, y + h, Vector2::new(uv_min.x, uv_max.y)),
            self.vertex(x + w, y + h, uv_max),
        ];

        Self::push_vertices(&mut self.triangle_batcher, &v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn push_textured_quad(&mut self, x: f32, y: f32, w: f32, h: f32, uv: math::Vector4<f32>) {
        let uv_min = uv.xy();
        let uv_max = Vector2::new(uv.x + uv.z, uv.y + uv.w);

        let v = [
            self.vertex(x, y, uv_min),                               // top-left
            self.vertex(x + w, y, Vector2::new(uv_max.x, uv_min.y)), // top-right
            self.vertex(x, y + h, Vector2::new(uv_min.x, uv_max.y)), // bottom-left
            self.vertex(x + w, y + h, uv_max),                       // bottom-right
        ];

        Self::push_vertices(&mut self.triangle_batcher, &v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn push_circle(&mut self, r: f32, x: f32, y: f32) {
        let center = Vector2::new(x, y);

        let v = [
            self.circle_vertex(x - r, y - r, center, r), // top-left
            self.circle_vertex(x + r, y - r, center, r), // top-right
            self.circle_vertex(x - r, y + r, center, r), // bottom-left
            self.circle_vertex(x + r, y + r, center, r), // bottom-right
        ];

        Self::push_vertices(&mut self.cirlce_batcher, &v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn present<'rp>(&mut self, rp: &mut wgpu::RenderPass<'rp>, pipelines: &PipelineCache) {
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
