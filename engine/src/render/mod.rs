pub mod camera;
pub mod color;
pub mod immediate;
pub mod layer;
pub mod packet;
pub mod retained;
pub mod vertex;

use gpu::Gpu;
use gpu::LayoutDesc;
use imgui::DrawData;
use imgui::ImguiRenderer;
use imgui::ImguiVertex;
use logging::warn;
use sdl3::gpu::ColorTargetInfo;
use sdl3::gpu::LoadOp;
use sdl3::gpu::StoreOp;

use crate::assets::Assets;
use crate::render::immediate::ImmediateRenderer;
use crate::render::packet::FramePacket;
use crate::render::vertex::Vertex;
use crate::window::platform::PlatformWindow;

const IMMEDIATE_VERT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/immediate.vert.spv"));
const IMMEDIATE_FRAG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/immediate.frag.spv"));
const IMGUI_VERT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/imgui.vert.spv"));
const IMGUI_FRAG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/imgui.frag.spv"));

const IMMEDIATE_SHADER: gpu::ShaderRef = gpu::ShaderRef::Builtin(0);
const IMGUI_SHADER: gpu::ShaderRef = gpu::ShaderRef::Builtin(1);

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

    gpu.load_shader(
        1,
        gpu::ShaderDesc {
            vertex_spirv: IMGUI_VERT,
            fragment_spirv: IMGUI_FRAG,
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

    fn imgui_desc(format: gpu::TextureFormat) -> gpu::PipelineDesc {
        gpu::PipelineDesc {
            shader: IMGUI_SHADER,
            vertex_layout: ImguiVertex::desc(),
            blend: gpu::BlendState::ALPHA_BLENDING,
            topology: gpu::PrimitiveTopology::TriangleList,
            // imgui emits both windings; culling would drop half of it.
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
        ui: Option<(&mut ImguiRenderer, DrawData)>,
    ) {
        let format = gpu.swapchain_format(window.inner());
        let immediate_desc = Pipelines::immediate_desc(format);
        let imgui_desc = Pipelines::imgui_desc(format);

        self.pipelines.ensure(gpu, &immediate_desc);
        self.pipelines.ensure(gpu, &imgui_desc);

        let Ok(mut cmd) = gpu.device.acquire_command_buffer() else {
            warn!("Failed to acquire command buffer for {:?}", window.id());
            return;
        };

        let Ok(swapchain) = cmd.wait_and_acquire_swapchain_texture(window.inner()) else {
            cmd.cancel();
            return;
        };

        let color_targets = [ColorTargetInfo::default()
            .with_texture(&swapchain)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_clear_color(packet.clear_color.into())];

        let Ok(copy_pass) = gpu.device.begin_copy_pass(&cmd) else {
            warn!("Failed to begin a copy pass for {}", window.id());
            cmd.cancel();
            return;
        };

        self.immediate.upload(gpu, &copy_pass, window, packet);

        let ui = ui.map(|(renderer, draw_data)| {
            renderer.upload(gpu, &copy_pass, window.inner(), draw_data);
            renderer
        });

        gpu.device.end_copy_pass(copy_pass);

        let Ok(rpass) = gpu.device.begin_render_pass(&cmd, &color_targets, None) else {
            warn!("Failed to begin a render pass for {}", window.id());
            cmd.cancel();
            return;
        };

        self.immediate.record(
            gpu,
            self.pipelines.get(&immediate_desc),
            &cmd,
            &rpass,
            window,
            assets,
            packet,
        );

        if let Some(renderer) = ui {
            renderer.record(
                gpu,
                self.pipelines.get(&imgui_desc),
                &cmd,
                &rpass,
                window.inner(),
            );
        }

        gpu.device.end_render_pass(rpass);

        if let Err(e) = cmd.submit() {
            warn!("Failed to submit the frame for {:?}: {}", window.id(), e);
        }
    }
}
