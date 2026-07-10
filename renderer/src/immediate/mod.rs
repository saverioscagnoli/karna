mod batcher;

use gpu::Vertex;
use logging::warn;

use crate::DrawCommand;
use crate::Layer;
use crate::Layouts;
use crate::immediate::batcher::Batcher;

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct RenderState {
    pub layer: Layer,
    pub draw_color: math::Vector4<f32>,
    pub transform: math::Matrix4<f32>,
    pub depth: f32,
}

pub struct ImmediateRenderer {
    triangle_batcher: Batcher<Vertex>,
}

impl ImmediateRenderer {
    pub fn new() -> Self {
        Self {
            triangle_batcher: Batcher::new(),
        }
    }

    fn push_vertices<V: Copy>(batcher: &mut Batcher<V>, vertices: &[V], pattern: &[u32]) {
        let base = batcher.vertex_count();

        batcher.vertices.extend_from_slice(vertices);
        batcher.indices.extend(pattern.iter().map(|i| base + i));
    }

    fn tesselate(&mut self, commands: &[DrawCommand]) {
        for command in commands {
            match *command {
                DrawCommand::ImmediateRect { x, y, w, h, state } => {
                    self.push_quad(x, y, w, h, state);
                }

                _ => {}
            }
        }
    }

    fn tp(&self, x: f32, y: f32, state: &RenderState) -> math::Vector3<f32> {
        let pos = math::Vector4::new(x, y, 0.0, 1.0);
        let t = state.transform.mul_vec(&pos);

        math::Vector3::new(t.x, t.y, state.depth)
    }

    fn vertex(&self, x: f32, y: f32, uv: math::Vector2<f32>, state: &RenderState) -> Vertex {
        let p = self.tp(x, y, state);
        Vertex::new(p, state.draw_color, uv)
    }

    fn push_quad(&mut self, x: f32, y: f32, w: f32, h: f32, state: RenderState) {
        let uv = math::Vector2::zero();

        let v = [
            self.vertex(x, y, uv, &state),
            self.vertex(x + w, y, uv, &state),
            self.vertex(x, y + h, uv, &state),
            self.vertex(x + w, y + h, uv, &state),
        ];

        Self::push_vertices(&mut self.triangle_batcher, &v, &[0, 1, 2, 2, 1, 3]);
    }

    pub fn present<'rp>(
        &'rp mut self,
        commands: &[DrawCommand],
        pass: &mut wgpu::RenderPass<'rp>,
        pipelines: &gpu::PipelineCache,
        layouts: &Layouts,
        format: wgpu::TextureFormat,
    ) {
        self.tesselate(commands);

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
