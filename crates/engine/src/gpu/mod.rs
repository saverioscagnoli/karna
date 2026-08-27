mod buffer;
mod info;
mod pipeline;
mod texture;
mod vertex;

use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ptr;
use std::sync::Arc;

use logging::debug;
use logging::fatal;
use logging::info;
use sdl3::SDL_AcquireGPUCommandBuffer;
use sdl3::SDL_BeginGPURenderPass;
use sdl3::SDL_CancelGPUCommandBuffer;
use sdl3::SDL_ClaimWindowForGPUDevice;
use sdl3::SDL_CreateGPUDevice;
use sdl3::SDL_DestroyGPUDevice;
use sdl3::SDL_EndGPURenderPass;
use sdl3::SDL_FColor;
use sdl3::SDL_GPU_SHADERFORMAT_SPIRV;
use sdl3::SDL_GPUColorTargetInfo;
use sdl3::SDL_GPUDevice;
use sdl3::SDL_GPULoadOp;
use sdl3::SDL_GPUStoreOp;
use sdl3::SDL_GPUTextureFormat;
use sdl3::SDL_GetGPUDeviceDriver;
use sdl3::SDL_GetGPUDeviceProperties;
use sdl3::SDL_GetGPUSwapchainTextureFormat;
use sdl3::SDL_SubmitGPUCommandBuffer;
use sdl3::SDL_WaitAndAcquireGPUSwapchainTexture;

use crate::Color;
use crate::err::sdl_last_error;
use crate::gpu::buffer::GpuTransferBuffer;
use crate::gpu::info::owned;
use crate::gpu::info::prop;
use crate::window::Window;

pub use crate::gpu::buffer::BufferError;
pub use crate::gpu::buffer::BufferUsage;
pub use crate::gpu::buffer::GpuBuffer;
pub use crate::gpu::buffer::Mapped;
pub use crate::gpu::info::GpuInfo;
pub use crate::gpu::pipeline::Blend;
pub use crate::gpu::pipeline::Cull;
pub use crate::gpu::pipeline::GraphicsPipeline;
pub use crate::gpu::pipeline::GraphicsPipelineDesc;
pub use crate::gpu::pipeline::Primitive;
pub use crate::gpu::pipeline::Shader;
pub use crate::gpu::pipeline::ShaderDesc;
pub use crate::gpu::pipeline::ShaderStage;
pub use crate::gpu::texture::Filter;
pub use crate::gpu::texture::Sampler;
pub use crate::gpu::texture::Texture;
pub use crate::gpu::texture::TextureDesc;
pub use crate::gpu::vertex::LayoutDescriptor;
pub use crate::gpu::vertex::VertexAttribute;
pub use crate::gpu::vertex::VertexLayout;

pub struct Gpu {
    device: *mut SDL_GPUDevice,
    staging: ManuallyDrop<RefCell<GpuTransferBuffer>>,
    info: GpuInfo,
}

impl Gpu {
    fn init(debug: bool) -> Self {
        unsafe {
            let device = SDL_CreateGPUDevice(SDL_GPU_SHADERFORMAT_SPIRV, debug, ptr::null());

            if device.is_null() {
                fatal!(
                    "Failed to initalize gpu device: {}\n\
                    The engine ships SPIR-V shaders, which requires the Vulkan backend.",
                    sdl_last_error()
                );
            }

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

            let info = GpuInfo {
                name,
                backend,
                driver,
            };

            debug!("GPU device initalized.");
            info!("{:?}", info);

            Self {
                device,
                staging: ManuallyDrop::new(RefCell::new(GpuTransferBuffer::new(device, 1024))),
                info,
            }
        }
    }

    pub fn raw(&self) -> *mut SDL_GPUDevice {
        self.device
    }

    pub fn claim_window(&self, window: &Window) {
        unsafe {
            if !SDL_ClaimWindowForGPUDevice(self.device, window.raw()) {
                fatal!(
                    "Failed to claim window for GPU device: {}",
                    sdl_last_error()
                );
            }
        }
    }

    pub fn swapchain_format(&self, window: &Window) -> SDL_GPUTextureFormat {
        unsafe { SDL_GetGPUSwapchainTextureFormat(self.device, window.raw()) }
    }

    pub fn clear(&self, window: &Window, color: Color) {
        unsafe {
            let cmd = SDL_AcquireGPUCommandBuffer(self.device);
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

            // Minimized or occluded: no texture, but the buffer still has to go
            // somewhere or you leak it.
            if swapchain.is_null() {
                SDL_SubmitGPUCommandBuffer(cmd);
                return;
            }

            let (r, g, b, a) = color.tuple();
            let target = SDL_GPUColorTargetInfo {
                texture: swapchain,
                clear_color: SDL_FColor { r, g, b, a },
                load_op: SDL_GPULoadOp::SDL_GPU_LOADOP_CLEAR,
                store_op: SDL_GPUStoreOp::SDL_GPU_STOREOP_STORE,
                ..Default::default()
            };

            let pass = SDL_BeginGPURenderPass(cmd, &target, 1, ptr::null());

            SDL_EndGPURenderPass(pass);
            SDL_SubmitGPUCommandBuffer(cmd);
        }
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        debug!("Dropping GPU device.");

        unsafe {
            ManuallyDrop::drop(&mut self.staging);
            SDL_DestroyGPUDevice(self.device);
        }
    }
}

#[derive(Clone)]
pub struct Device(Arc<Gpu>);

impl Deref for Device {
    type Target = Gpu;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Device {
    pub fn init(debug: bool) -> Self {
        Self(Arc::new(Gpu::init(debug)))
    }
}
