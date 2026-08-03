pub mod camera;
pub mod color;
pub mod geometry;
pub mod immediate;
pub mod layer;
pub mod material;
pub mod mesh;
pub mod packet;
pub mod retained;
pub mod transform;
pub mod vertex;

use gpu::DepthStencilTargetInfo;
use gpu::Gpu;
use gpu::LayoutDesc;
use imgui::DrawData;
use imgui::ImguiRenderer;
use imgui::ImguiVertex;
use logging::warn;
use sdl3::gpu::ColorTargetInfo;
use sdl3::gpu::LoadOp;
use sdl3::gpu::StoreOp;
use utils::FastHashMap;
use utils::WindowId;

use crate::assets::Assets;
use crate::render::immediate::ImmediateRenderer;
use crate::render::layer::Layer;
use crate::render::mesh::MeshRenderer;
use crate::render::packet::FramePacket;
use crate::render::retained::RenderWorld;
use crate::render::vertex::Vertex;
use crate::window::platform::PlatformWindow;

const IMMEDIATE_VERT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/immediate.vert.spv"));
const IMMEDIATE_FRAG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/immediate.frag.spv"));
const IMGUI_VERT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/imgui.vert.spv"));
const IMGUI_FRAG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/imgui.frag.spv"));
const MESH_VERT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mesh.vert.spv"));
const MESH_FRAG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mesh.frag.spv"));

const IMMEDIATE_SHADER: gpu::ShaderRef = gpu::ShaderRef::Builtin(0);
const IMGUI_SHADER: gpu::ShaderRef = gpu::ShaderRef::Builtin(1);
pub const MESH_SHADER: gpu::ShaderRef = gpu::ShaderRef::Builtin(2);

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

    gpu.load_shader(
        2,
        gpu::ShaderDesc {
            vertex_spirv: MESH_VERT,
            fragment_spirv: MESH_FRAG,
            vertex_uniform_buffers: 2,
            fragment_samplers: 1,
            fragment_uniform_buffers: 1,
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
            depth: None,
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
            depth: None,
        }
    }
}

pub struct Renderer {
    pipelines: gpu::PipelineCache,
    immediate: ImmediateRenderer,
    meshes: MeshRenderer,
    depth: FastHashMap<WindowId, gpu::Texture>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            pipelines: gpu::PipelineCache::new(),
            immediate: ImmediateRenderer::new(),
            meshes: MeshRenderer::new(),
            depth: FastHashMap::default(),
        }
    }

    fn depth_target(&mut self, gpu: &Gpu, window: &PlatformWindow) -> &mut gpu::Texture {
        let size = window.pixel_size();
        let id = window.id();

        let stale = self
            .depth
            .get(&id)
            .map(|t| t.width() != size.width || t.height() != size.height)
            .unwrap_or(true);

        if stale {
            let texture =
                gpu::Texture::depth(gpu, format!("{id:?}-depth"), size.width, size.height);

            self.depth.insert(id, texture);
        }

        self.depth.get_mut(&id).expect("depth target just inserted")
    }

    pub fn present(
        &mut self,
        gpu: &Gpu,
        window: &PlatformWindow,
        assets: &Assets,
        packet: &FramePacket,
        render: &mut RenderWorld,
        ui: Option<(&mut ImguiRenderer, DrawData)>,
    ) {
        let format = gpu.swapchain_format(window.inner());
        let immediate_desc = Pipelines::immediate_desc(format);
        let imgui_desc = Pipelines::imgui_desc(format);

        self.pipelines.ensure(gpu, &immediate_desc);
        self.pipelines.ensure(gpu, &imgui_desc);

        let world = &packet[Layer::World];

        // Build a pipeline per distinct mesh pass. The draw list is sorted with
        // pass_id in the high bits, so tracking the previous one collapses this
        // to one `ensure` per pass rather than one per draw.
        let mut last_pass = u16::MAX;

        for item in &world.meshes {
            let material = render.materials.get(item.material);

            if material.pass_id == last_pass {
                continue;
            }

            if let Some(geometry) = render.geometries.get(item.geometry) {
                let desc = render
                    .materials
                    .pass(material.pass_id)
                    .pipeline_desc(geometry.layout, format);

                self.pipelines.ensure(gpu, &desc);
                last_pass = material.pass_id;
            }
        }

        let Ok(mut cmd) = gpu.device.acquire_command_buffer() else {
            warn!("Failed to acquire command buffer for {:?}", window.id());
            return;
        };

        let Ok(swapchain) = cmd.wait_and_acquire_swapchain_texture(window.inner()) else {
            cmd.cancel();
            return;
        };

        let clear_targets = [ColorTargetInfo::default()
            .with_texture(&swapchain)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_clear_color(render.clear_color.into())];

        let load_targets = [ColorTargetInfo::default()
            .with_texture(&swapchain)
            .with_load_op(LoadOp::LOAD)
            .with_store_op(StoreOp::STORE)];

        let Ok(copy_pass) = gpu.device.begin_copy_pass(&cmd) else {
            warn!("Failed to begin a copy pass for {}", window.id());
            cmd.cancel();
            return;
        };

        self.immediate.upload(gpu, &copy_pass, window, packet);
        render.geometries.upload_dirty(gpu, &copy_pass);

        let ui = ui.map(|(renderer, draw_data)| {
            renderer.upload(gpu, &copy_pass, window.inner(), draw_data);
            renderer
        });

        gpu.device.end_copy_pass(copy_pass);

        let drew_meshes = !world.meshes.is_empty();

        if drew_meshes {
            let depth_info = {
                let depth = self.depth_target(gpu, window);

                DepthStencilTargetInfo::new()
                    .with_texture(depth.raw_mut())
                    .with_clear_depth(1.0)
                    .with_load_op(LoadOp::CLEAR)
                    .with_store_op(StoreOp::DONT_CARE)
            };

            match gpu
                .device
                .begin_render_pass(&cmd, &clear_targets, Some(&depth_info))
            {
                Ok(rpass) => {
                    self.meshes.record(
                        gpu,
                        &self.pipelines,
                        &cmd,
                        &rpass,
                        assets,
                        &render.materials,
                        &render.geometries,
                        &world.camera,
                        format,
                        &world.meshes,
                    );

                    gpu.device.end_render_pass(rpass);
                }

                Err(_) => warn!("Failed to begin the mesh pass for {}", window.id()),
            }
        }

        let color_targets = if drew_meshes {
            &load_targets
        } else {
            &clear_targets
        };

        let Ok(rpass) = gpu.device.begin_render_pass(&cmd, color_targets, None) else {
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
