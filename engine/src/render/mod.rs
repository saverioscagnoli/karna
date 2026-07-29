pub mod camera;
pub mod color;
pub mod immediate;
pub mod layer;
pub mod packet;
pub mod retained;
pub mod vertex;

use gpu::Gpu;

use crate::assets::Assets;
use crate::render::immediate::ImmediateRenderer;
use crate::render::packet::FramePacket;
use crate::render::vertex::LayoutDesc;
use crate::render::vertex::Vertex;
use crate::window::platform::PlatformWindow;

const IMMEDIATE_VERT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/immediate.vert.spv"));
const IMMEDIATE_FRAG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/immediate.frag.spv"));

const IMMEDIATE_SHADER: gpu::ShaderRef = gpu::ShaderRef::Builtin(0);

pub(crate) fn load_builtin_shaders(gpu: &mut Gpu) {
    gpu.load_shader(
        0,
        gpu::ShaderDesc {
            vertex_spirv: IMMEDIATE_VERT,
            fragment_spirv: IMMEDIATE_FRAG,
            vertex_uniform_buffers: 1,
            fragment_samplers: 1,
            fragment_uniform_buffers: 0,
        },
    );
}

struct Pipelines;

impl Pipelines {
    fn immediate_desc(format: gpu::TextureFormat) -> gpu::PipelineDesc {
        gpu::PipelineDesc {
            shader: IMMEDIATE_SHADER,
            vertex_layout: Vertex::desc(),
            blend: gpu::BlendState::ALPHA_BLENDING,
            topology: gpu::PrimitiveTopology::TriangleList,
            cull: None,
            format,
        }
    }
}

pub struct Renderer {
    pipelines: gpu::PipelineCache,
    immediate: ImmediateRenderer,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            pipelines: gpu::PipelineCache::new(),
            immediate: ImmediateRenderer::new(),
        }
    }

    pub fn present(
        &mut self,
        gpu: &Gpu,
        window: &PlatformWindow,
        assets: &Assets,
        packet: &FramePacket,
    ) {
        self.immediate
            .present(gpu, &mut self.pipelines, window, assets, packet);
    }
}
