pub mod camera;
pub mod color;
pub mod immediate;
pub mod layer;
pub mod packet;
pub mod retained;
pub mod target;
pub mod vertex;
pub mod view;

use gpu::Gpu;
use logging::warn;
use sdl3::gpu::{ColorTargetInfo, LoadOp, StoreOp};

use crate::render::layer::{Layer, LayerGPU};
use crate::render::packet::FramePacket;
use crate::render::vertex::{Vertex, VertexLayoutDescriptor};
use crate::window::SdlWindow;

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
            fragment_samplers: 0,
            fragment_uniform_buffers: 0,
        },
    );
}

pub struct Renderer {
    format: gpu::TextureFormat,
    pipelines: gpu::PipelineCache,
    layers: [LayerGPU; 3],
}

impl Renderer {
    pub(crate) fn new(gpu: &Gpu, format: gpu::TextureFormat) -> Self {
        let mut pipelines = gpu::PipelineCache::new();
        pipelines.create(gpu, &Self::immediate_desc(format));

        Self {
            format,
            pipelines,
            layers: [LayerGPU::new(gpu), LayerGPU::new(gpu), LayerGPU::new(gpu)],
        }
    }

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

    pub(crate) fn present(&mut self, gpu: &Gpu, window: &SdlWindow, packet: &FramePacket) {
        let Ok(mut cmd) = gpu.device.acquire_command_buffer() else {
            warn!("Failed to acquire command buffer");
            return;
        };

        let Ok(copy_pass) = gpu.device.begin_copy_pass(&cmd) else {
            warn!("Failed to begin copy pass");
            cmd.cancel();
            return;
        };

        for l in Layer::ALL {
            let data = &packet[l].data;

            if data.indices.is_empty() {
                continue;
            }

            let buffers = &mut self.layers[l as usize];
            buffers.vertex.write_all(gpu, &copy_pass, &data.vertices);
            buffers.index.write_all(gpu, &copy_pass, &data.indices);
        }

        gpu.device.end_copy_pass(copy_pass);

        let Ok(swapchain) = cmd.wait_and_acquire_swapchain_texture(window) else {
            cmd.cancel();
            return;
        };

        let targets = [ColorTargetInfo::default()
            .with_texture(&swapchain)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_clear_color(packet.clear_color.into())];

        let pass = gpu
            .device
            .begin_render_pass(&cmd, &targets, None)
            .expect("Failed to begin render pass");

        for l in Layer::ALL {
            let data = &packet[l].data;

            if data.indices.is_empty() {
                continue;
            }

            let buffers = &self.layers[l as usize];

            pass.bind_graphics_pipeline(self.pipelines.get(&Self::immediate_desc(self.format)));
            cmd.push_vertex_uniform_data(0, &packet[l].camera);
            pass.bind_vertex_buffers(0, &[buffers.vertex.binding()]);
            pass.bind_index_buffer(&buffers.index.binding(), gpu::IndexElementSize::_32BIT);
            pass.draw_indexed_primitives(data.indices.len() as u32, 1, 0, 0, 0);
        }

        gpu.device.end_render_pass(pass);

        if let Err(e) = cmd.submit() {
            warn!("Failed to submit frame: {e}");
        }
    }
}
