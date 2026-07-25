mod buffer;
mod pipeline;
mod shaders;
mod vertex;

use std::ffi::CStr;

use logging::{debug, info};
use sdl3::{
    gpu::{Device, Sampler, ShaderFormat},
    sys::{
        gpu::{
            SDL_ClaimWindowForGPUDevice, SDL_GetGPUDeviceDriver, SDL_GetGPUDeviceProperties,
            SDL_PROP_GPU_DEVICE_DRIVER_INFO_STRING, SDL_PROP_GPU_DEVICE_NAME_STRING,
            SDL_ReleaseWindowFromGPUDevice,
        },
        properties::SDL_GetStringProperty,
    },
    video::Window,
};

pub use sdl3::gpu::CommandBuffer;
pub use sdl3::gpu::CopyPass;
pub use sdl3::gpu::IndexElementSize;
pub use sdl3::gpu::PrimitiveType as PrimitiveTopology;
pub use sdl3::gpu::RenderPass;
pub use sdl3::gpu::TextureFormat;
pub use sdl3::gpu::TextureSamplerBinding;

pub use crate::buffer::*;
pub use crate::pipeline::*;
pub use crate::shaders::*;
pub use crate::vertex::*;

pub struct Gpu {
    pub device: Device,
    pub sampler: Sampler,
    pub shaders: ShaderRegistry,
}

impl Gpu {
    pub fn init() -> Self {
        let device =
            Device::new(ShaderFormat::SPIRV, cfg!(debug_assertions)).expect("Failed to init gpu");

        let sampler = device
            .create_sampler(
                sdl3::gpu::SamplerCreateInfo::new()
                    .with_min_filter(sdl3::gpu::Filter::Nearest)
                    .with_mag_filter(sdl3::gpu::Filter::Nearest)
                    .with_mipmap_mode(sdl3::gpu::SamplerMipmapMode::Nearest)
                    .with_address_mode_u(sdl3::gpu::SamplerAddressMode::ClampToEdge)
                    .with_address_mode_v(sdl3::gpu::SamplerAddressMode::ClampToEdge)
                    .with_address_mode_w(sdl3::gpu::SamplerAddressMode::ClampToEdge),
            )
            .expect("Failed to create sampler");

        Self {
            device,
            sampler,
            shaders: ShaderRegistry::new(),
        }
    }

    pub fn swapchain_format(&self, window: &Window) -> TextureFormat {
        self.device.get_swapchain_texture_format(window)
    }

    pub fn load_shader(&mut self, index: usize, desc: ShaderDesc) {
        let Self {
            device, shaders, ..
        } = self;

        shaders.load(device, index, desc);
    }

    pub fn log_shader_formats(&self) {
        debug!(
            "GPU device created (formats: {:?})",
            self.device.get_shader_formats()
        );
    }

    pub fn log_info(&self) {
        unsafe {
            let raw = self.device.raw();

            let backend = CStr::from_ptr(SDL_GetGPUDeviceDriver(raw)).to_string_lossy();

            let props = SDL_GetGPUDeviceProperties(raw); // 0 on failure
            let unknown = c"unknown".as_ptr();
            let name = CStr::from_ptr(SDL_GetStringProperty(
                props,
                SDL_PROP_GPU_DEVICE_NAME_STRING,
                unknown,
            ))
            .to_string_lossy();
            let driver = CStr::from_ptr(SDL_GetStringProperty(
                props,
                SDL_PROP_GPU_DEVICE_DRIVER_INFO_STRING,
                unknown,
            ))
            .to_string_lossy();

            info!("GPU: {} (backend {}, driver {})", name, backend, driver);
        }
    }

    pub fn claim_window(&self, window: &Window) -> Result<(), String> {
        let ok = unsafe { SDL_ClaimWindowForGPUDevice(self.device.raw(), window.raw()) };
        if ok {
            Ok(())
        } else {
            Err(sdl3::get_error().to_string())
        }
    }

    pub fn release_window(&self, window: &Window) {
        unsafe { SDL_ReleaseWindowFromGPUDevice(self.device.raw(), window.raw()) };
    }
}
