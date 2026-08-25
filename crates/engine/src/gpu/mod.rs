pub mod buffer;
pub mod pipeline;
pub mod present_mode;
pub mod texture;
pub mod vertex;

use std::cell::RefCell;
use std::ffi::CStr;
use std::ffi::c_char;
use std::ffi::c_void;
use std::mem;
use std::mem::ManuallyDrop;
use std::ptr;

use logging::debug;
use logging::fatal;
use sdl3::SDL_AcquireGPUCommandBuffer;
use sdl3::SDL_BeginGPURenderPass;
use sdl3::SDL_BindGPUFragmentSamplers;
use sdl3::SDL_BindGPUGraphicsPipeline;
use sdl3::SDL_BindGPUIndexBuffer;
use sdl3::SDL_BindGPUVertexBuffers;
use sdl3::SDL_CreateGPUDevice;
use sdl3::SDL_DestroyGPUDevice;
use sdl3::SDL_DrawGPUIndexedPrimitives;
use sdl3::SDL_EndGPURenderPass;
use sdl3::SDL_FColor;
use sdl3::SDL_GPU_SHADERFORMAT_SPIRV;
use sdl3::SDL_GPUBufferBinding;
use sdl3::SDL_GPUColorTargetInfo;
use sdl3::SDL_GPUDevice;
use sdl3::SDL_GPUIndexElementSize;
use sdl3::SDL_GPULoadOp;
use sdl3::SDL_GPUStoreOp;
use sdl3::SDL_GPUTexture;
use sdl3::SDL_GPUTextureSamplerBinding;
use sdl3::SDL_GetGPUDeviceDriver;
use sdl3::SDL_GetGPUDeviceProperties;
use sdl3::SDL_GetGPUSwapchainTextureFormat;
use sdl3::SDL_GetStringProperty;
use sdl3::SDL_PropertiesID;
use sdl3::SDL_PushGPUVertexUniformData;
use sdl3::SDL_SubmitGPUCommandBuffer;
use sdl3::SDL_WaitAndAcquireGPUSwapchainTexture;

use crate::err::SDL_LastError;
use crate::gpu::buffer::GpuTransferBuffer;
use crate::gpu::pipeline::DrawCall;
use crate::gpu::pipeline::Pipeline;
use crate::render::color::Color;
use crate::window::platform::PlatformWindow;

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub backend: String,
    pub driver: String,
}

/// Copies a borrowed C string. SDL owns the memory, so this must not escape
/// as a pointer — copy immediately.
unsafe fn owned(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

unsafe fn prop(props: SDL_PropertiesID, key: &CStr) -> Option<String> {
    if props == 0 {
        return None;
    }

    unsafe { owned(SDL_GetStringProperty(props, key.as_ptr(), ptr::null())) }
}

pub struct Gpu {
    device: *mut SDL_GPUDevice,
    staging: ManuallyDrop<RefCell<GpuTransferBuffer>>,
    pipeline: ManuallyDrop<RefCell<Option<Pipeline>>>,
    info: GpuInfo,
}

impl Gpu {
    // Do not call before SDL_INIT
    pub fn init() -> Self {
        unsafe {
            // Only claim the formats we can actually feed SDL. build.rs
            // compiles the shaders with glslc, so SPIR-V is all we ship —
            // advertising DXIL/MSL just lets SDL pick D3D12 or Metal and
            // then trip an assert inside SDL_CreateGPUShader.
            let device = SDL_CreateGPUDevice(SDL_GPU_SHADERFORMAT_SPIRV, true, ptr::null());

            if device.is_null() {
                fatal!(
                    "Failed to initalize gpu device: {}\n\
                     The engine ships SPIR-V shaders, which requires the Vulkan backend.",
                    SDL_LastError()
                );
            }

            debug!("GPU Device initialized.");

            let backend = owned(SDL_GetGPUDeviceDriver(device)).unwrap_or(String::from("unknown"));
            let props = SDL_GetGPUDeviceProperties(device);
            let name = prop(props, c"SDL.gpu.device.name").unwrap_or(String::from("unknown"));
            let driver_name = prop(props, c"SDL.gpu.device.driver_name");
            let driver_version = prop(props, c"SDL.gpu.device.driver_version");

            let driver = match (driver_name, driver_version) {
                (Some(n), Some(v)) => format!("{} {}", n, v),
                (Some(n), None) => n,
                (None, Some(v)) => v,
                (None, None) => String::from("unknown"),
            };

            Self {
                device,
                staging: ManuallyDrop::new(RefCell::new(GpuTransferBuffer::new(device, 1024))),
                pipeline: ManuallyDrop::new(RefCell::new(None)),
                info: GpuInfo {
                    name,
                    backend,
                    driver,
                },
            }
        }
    }

    pub fn raw(&self) -> *mut SDL_GPUDevice {
        self.device
    }

    pub fn info(&self) -> &GpuInfo {
        &self.info
    }

    fn pipeline_for(&self, window: &PlatformWindow) {
        let mut pipeline = self.pipeline.borrow_mut();

        if pipeline.is_none() {
            let format = unsafe { SDL_GetGPUSwapchainTextureFormat(self.device, window.raw()) };
            *pipeline = Some(Pipeline::new(self.device, format));
        }
    }

    pub(crate) fn white_texture(&self, window: &PlatformWindow) -> *mut SDL_GPUTexture {
        self.pipeline_for(window);
        let pipeline = self.pipeline.borrow();
        pipeline.as_ref().expect("pipeline initialized above").white_texture()
    }

    pub(crate) fn render(&self, window: &PlatformWindow, color: Color, calls: &[DrawCall]) {
        self.pipeline_for(window);
        let pipeline = self.pipeline.borrow();
        let pipeline = pipeline.as_ref().expect("pipeline initialized above");

        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.device);

            if cmd.is_null() {
                fatal!("Failed to acquire command buffer: {}", SDL_LastError());
            }

            let mut swapchain: *mut SDL_GPUTexture = ptr::null_mut();
            let ok = SDL_WaitAndAcquireGPUSwapchainTexture(
                cmd,
                window.raw(),
                &mut swapchain,
                ptr::null_mut(), // width
                ptr::null_mut(), // height
            );

            if !ok {
                fatal!("Failed to acquire swapchain texture: {}", SDL_LastError());
            }

            if swapchain.is_null() {
                SDL_SubmitGPUCommandBuffer(cmd);
                return;
            }

            let (r, g, b, a) = color.tuple();
            let mut target: SDL_GPUColorTargetInfo = mem::zeroed();

            target.texture = swapchain;
            target.clear_color = SDL_FColor { r, g, b, a };
            target.load_op = SDL_GPULoadOp::SDL_GPU_LOADOP_CLEAR;
            target.store_op = SDL_GPUStoreOp::SDL_GPU_STOREOP_STORE;

            let pass = SDL_BeginGPURenderPass(cmd, &target, 1, ptr::null());

            if !calls.is_empty() {
                SDL_BindGPUGraphicsPipeline(pass, pipeline.raw());

                for call in calls {
                    let mvp = call.mvp;
                    let bytes = mvp.as_bytes();

                    SDL_PushGPUVertexUniformData(
                        cmd,
                        0,
                        bytes.as_ptr() as *const c_void,
                        bytes.len() as u32,
                    );

                    let sampler_binding = SDL_GPUTextureSamplerBinding {
                        texture: call.texture,
                        sampler: pipeline.white_sampler(),
                    };
                    SDL_BindGPUFragmentSamplers(pass, 0, &sampler_binding, 1);

                    let vertex_binding = SDL_GPUBufferBinding {
                        buffer: call.vertex_buffer,
                        offset: 0,
                    };
                    SDL_BindGPUVertexBuffers(pass, 0, &vertex_binding, 1);

                    let index_binding = SDL_GPUBufferBinding {
                        buffer: call.index_buffer,
                        offset: 0,
                    };
                    SDL_BindGPUIndexBuffer(
                        pass,
                        &index_binding,
                        SDL_GPUIndexElementSize::SDL_GPU_INDEXELEMENTSIZE_32BIT,
                    );

                    SDL_DrawGPUIndexedPrimitives(pass, call.num_indices, 1, call.first_index, 0, 0);
                }
            }

            SDL_EndGPURenderPass(pass);

            if !SDL_SubmitGPUCommandBuffer(cmd) {
                fatal!("Failed to submit command buffer: {}", SDL_LastError());
            }
        }
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        debug!("Dropping GPU device.");

        unsafe {
            ManuallyDrop::drop(&mut self.pipeline);
            ManuallyDrop::drop(&mut self.staging);
            SDL_DestroyGPUDevice(self.device);
        }
    }
}
