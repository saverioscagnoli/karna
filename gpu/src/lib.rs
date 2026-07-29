mod buffer;
mod pipeline;
mod shaders;
mod texture;
mod vertex;

use std::ffi::CStr;

use logging::debug;
use logging::info;
use logging::warn;
use sdl3::gpu::ColorTargetInfo;
use sdl3::gpu::Device;
use sdl3::gpu::LoadOp;
use sdl3::gpu::Sampler;
use sdl3::gpu::ShaderFormat;
use sdl3::gpu::StoreOp;
use sdl3::gpu::SwapchainComposition;
use sdl3::pixels::Color;
use sdl3::sys::gpu::SDL_ClaimWindowForGPUDevice;
use sdl3::sys::gpu::SDL_GPUPresentMode;
use sdl3::sys::gpu::SDL_GetGPUDeviceDriver;
use sdl3::sys::gpu::SDL_GetGPUDeviceProperties;
use sdl3::sys::gpu::SDL_PROP_GPU_DEVICE_DRIVER_INFO_STRING;
use sdl3::sys::gpu::SDL_PROP_GPU_DEVICE_NAME_STRING;
use sdl3::sys::gpu::SDL_ReleaseWindowFromGPUDevice;
use sdl3::sys::gpu::SDL_WindowSupportsGPUPresentMode;
use sdl3::sys::properties::SDL_GetStringProperty;
use sdl3::video::Window;

pub use sdl3::gpu::CommandBuffer;
pub use sdl3::gpu::CopyPass;
pub use sdl3::gpu::IndexElementSize;
pub use sdl3::gpu::PresentMode;
pub use sdl3::gpu::PrimitiveType as PrimitiveTopology;
pub use sdl3::gpu::RenderPass;
pub use sdl3::gpu::TextureFormat;
pub use sdl3::gpu::TextureSamplerBinding;

pub use crate::buffer::*;
pub use crate::pipeline::*;
pub use crate::shaders::*;
pub use crate::texture::*;
pub use crate::vertex::*;

pub struct Gpu {
    pub device: Device,
    pub sampler: Sampler,
    pub shaders: ShaderRegistry,
    present_mode: PresentMode,
}

impl Gpu {
    pub fn init() -> Result<Self, String> {
        let device =
            Device::new(ShaderFormat::SPIRV, cfg!(debug_assertions)).map_err(|e| e.to_string())?;

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
            .map_err(|e| e.to_string())?;

        Ok(Self {
            device,
            sampler,
            shaders: ShaderRegistry::new(),
            present_mode: PresentMode::Vsync,
        })
    }

    pub fn supports_present_mode(&self, window: &Window, mode: PresentMode) -> bool {
        let mode = match mode {
            PresentMode::Immediate => SDL_GPUPresentMode::IMMEDIATE,
            PresentMode::Vsync => SDL_GPUPresentMode::VSYNC,
            PresentMode::Mailbox => SDL_GPUPresentMode::MAILBOX,
        };

        unsafe { SDL_WindowSupportsGPUPresentMode(self.device.raw(), window.raw(), mode) }
    }

    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    pub fn set_present_mode(&mut self, window: &Window, mode: PresentMode) {
        if let Err(e) =
            self.device
                .set_swapchain_parameters(window, mode, SwapchainComposition::default())
        {
            warn!("Failed to set present mode: {}", e);
            return;
        }

        debug!("Present mode set to {:?}", mode);
        self.present_mode = mode;
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

            info!("GPU: {} (backend: {}, driver: {})", name, backend, driver);
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

    pub fn clear(&self, window: &Window, color: Color) -> Result<(), sdl3::Error> {
        let mut cmd = self.device.acquire_command_buffer()?;

        // Returns an error / null texture when the window is minimized or
        // the swapchain isn't ready. Don't treat that as fatal.
        let Ok(swapchain) = cmd.wait_and_acquire_swapchain_texture(window) else {
            cmd.cancel();
            return Ok(());
        };

        let targets = [ColorTargetInfo::default()
            .with_texture(&swapchain)
            .with_load_op(LoadOp::CLEAR)
            .with_store_op(StoreOp::STORE)
            .with_clear_color(color)];

        let pass = self.device.begin_render_pass(&cmd, &targets, None)?;
        self.device.end_render_pass(pass);

        cmd.submit()?;
        Ok(())
    }
}
