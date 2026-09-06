use std::ptr;

use logging::debug;
use math as m;
use sdl3::SDL_AcquireGPUCommandBuffer;
use sdl3::SDL_BeginGPURenderPass;
use sdl3::SDL_BindGPUFragmentSamplers;
use sdl3::SDL_BindGPUGraphicsPipeline;
use sdl3::SDL_BindGPUIndexBuffer;
use sdl3::SDL_BindGPUVertexBuffers;
use sdl3::SDL_CancelGPUCommandBuffer;
use sdl3::SDL_DrawGPUIndexedPrimitives;
use sdl3::SDL_EndGPURenderPass;
use sdl3::SDL_FColor;
use sdl3::SDL_GPUBufferBinding;
use sdl3::SDL_GPUColorTargetInfo;
use sdl3::SDL_GPUIndexElementSize;
use sdl3::SDL_GPULoadOp;
use sdl3::SDL_GPUStoreOp;
use sdl3::SDL_PushGPUVertexUniformData;
use sdl3::SDL_SubmitGPUCommandBuffer;
use sdl3::SDL_WaitAndAcquireGPUSwapchainTexture;

use crate::Color;
use crate::assets::AssetServer;
use crate::err::sdl_last_error;
use crate::gpu::Device;
use crate::gpu::GraphicsPipeline;
use crate::render::Camera;
use crate::render::Projection;
use crate::render::draw::Draw;
use crate::render::geometry::ImmediateGeometry;
use crate::render::layer::LayerData;
use crate::render::layer::LayerMap;
use crate::render::pipeline::PipelineCache;
use crate::render::pipeline::SamplerCache;
use crate::window::Window;

pub struct Renderer {
    device: Device,
    cameras: LayerMap<Camera>,
    data: LayerMap<LayerData>,
    pipelines: PipelineCache,
    samplers: SamplerCache,
}

impl Renderer {
    pub fn new(device: Device, viewport: m::Size<u32>) -> Self {
        let default_camera = Camera::new(Projection::topleft_2d(viewport));
        let cameras = LayerMap::new(default_camera, default_camera, default_camera);

        let data = LayerMap::new(
            LayerData::ThreeDimensional {
                immediate: ImmediateGeometry::new(
                    device.clone(),
                    "World layer immediate buffers",
                    Vec::new(),
                    Vec::new(),
                ),
            },
            LayerData::TwoDimensional {
                immediate: ImmediateGeometry::new(
                    device.clone(),
                    "Ui layer immediate buffers",
                    Vec::new(),
                    Vec::new(),
                ),
            },
            LayerData::TwoDimensional {
                immediate: ImmediateGeometry::new(
                    device.clone(),
                    "Debug layer immediate buffers",
                    Vec::new(),
                    Vec::new(),
                ),
            },
        );

        Self {
            device,
            cameras,
            data,
            pipelines: PipelineCache::default(),
            samplers: SamplerCache::default(),
        }
    }

    pub fn draw<'w>(&'w mut self, viewport: m::Size<u32>, assets: &'w mut AssetServer) -> Draw<'w> {
        Draw::new(&self.cameras, &mut self.data, viewport, assets)
    }

    pub fn immediate_pipeline(&mut self, window: &Window) -> &GraphicsPipeline {
        let format = self.device.swapchain_format(window);

        self.pipelines.immediate(&self.device, format)
    }

    pub(crate) fn begin_frame(&mut self, viewport: m::Size<u32>, assets: &mut AssetServer) {
        assets.begin_frame();

        for index in 0..self.data.len() {
            let layer = self.data.layer_at(index);

            self.cameras[layer].update(viewport);
            self.data[layer].immediate_mut().clear();
        }
    }

    pub(crate) fn render(&mut self, window: &Window, assets: &AssetServer, clear: Color) {
        let format = self.device.swapchain_format(window);

        for index in 0..self.data.len() {
            let layer = self.data.layer_at(index);
            let geometry = self.data[layer].immediate_mut();

            if geometry.is_empty() {
                continue;
            }

            #[rustfmt::skip]
            let ImmediateGeometry { vertices, indices, vertex_buffer, index_buffer, .. } = geometry;

            if let Err(e) = self.device.upload(vertex_buffer, vertices) {
                debug!("Failed to upload immediate vertices: {:?}", e);
                continue;
            }

            if let Err(e) = self.device.upload(index_buffer, indices) {
                debug!("Failed to upload immediate indices: {:?}", e);
                continue;
            }
        }

        let pipeline = self.pipelines.immediate(&self.device, format);

        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.device.raw());

            if cmd.is_null() {
                debug!("Failed to acquire command buffer: {}", sdl_last_error());
                return;
            }

            let mut swapchain = ptr::null_mut();

            if !SDL_WaitAndAcquireGPUSwapchainTexture(
                cmd,
                window.raw(),
                &mut swapchain,
                ptr::null_mut(),
                ptr::null_mut(),
            ) {
                SDL_CancelGPUCommandBuffer(cmd);
                return;
            }

            if swapchain.is_null() {
                SDL_SubmitGPUCommandBuffer(cmd);
                return;
            }

            let (r, g, b, a) = clear.tuple();
            let target = SDL_GPUColorTargetInfo {
                texture: swapchain,
                clear_color: SDL_FColor { r, g, b, a },
                load_op: SDL_GPULoadOp::SDL_GPU_LOADOP_CLEAR,
                store_op: SDL_GPUStoreOp::SDL_GPU_STOREOP_STORE,
                ..Default::default()
            };

            let pass = SDL_BeginGPURenderPass(cmd, &target, 1, ptr::null());

            SDL_BindGPUGraphicsPipeline(pass, pipeline.raw());

            for index in 0..self.data.len() {
                let layer = self.data.layer_at(index);
                let geometry = self.data[layer].immediate();

                if geometry.is_empty() {
                    continue;
                }

                let mvp = self.cameras[layer].mvp();
                let bytes = mvp.as_bytes();

                SDL_PushGPUVertexUniformData(cmd, 0, bytes.as_ptr().cast(), bytes.len() as u32);

                let vertices = SDL_GPUBufferBinding {
                    buffer: geometry.vertex_buffer.raw(),
                    offset: 0,
                };

                let indices = SDL_GPUBufferBinding {
                    buffer: geometry.index_buffer.raw(),
                    offset: 0,
                };

                SDL_BindGPUVertexBuffers(pass, 0, &vertices, 1);
                SDL_BindGPUIndexBuffer(
                    pass,
                    &indices,
                    SDL_GPUIndexElementSize::SDL_GPU_INDEXELEMENTSIZE_32BIT,
                );

                for batch in &geometry.batches {
                    let Some(texture) = assets.page_texture(batch.page) else {
                        continue;
                    };

                    let filter = assets.page_filter(batch.page).unwrap_or_default();
                    let sampler = self.samplers.get(&self.device, filter);
                    let binding = texture.binding(sampler);

                    SDL_BindGPUFragmentSamplers(pass, 0, &binding, 1);
                    SDL_DrawGPUIndexedPrimitives(pass, batch.count, 1, batch.start, 0, 0);
                }
            }

            SDL_EndGPURenderPass(pass);
            SDL_SubmitGPUCommandBuffer(cmd);
        }
    }
}
