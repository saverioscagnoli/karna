use logging::debug;
use sdl3::SDL_GPUPresentMode;
use sdl3::SDL_GPUSwapchainComposition;
use sdl3::SDL_SetGPUSwapchainParameters;
use sdl3::SDL_WindowSupportsGPUPresentMode;

use crate::err::sdl_last_error;
use crate::gpu::Gpu;
use crate::window::Window;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentMode(SDL_GPUPresentMode);

impl PresentMode {
    pub const IMMEDIATE: Self = Self(SDL_GPUPresentMode::SDL_GPU_PRESENTMODE_IMMEDIATE);
    pub const MAILBOX: Self = Self(SDL_GPUPresentMode::SDL_GPU_PRESENTMODE_MAILBOX);
    pub const VSYNC: Self = Self(SDL_GPUPresentMode::SDL_GPU_PRESENTMODE_VSYNC);
}
impl Gpu {
    pub fn set_present_mode(&self, window: &Window, mode: PresentMode) -> bool {
        unsafe {
            if !SDL_WindowSupportsGPUPresentMode(self.device, window.raw(), mode.0) {
                debug!("Present mode {:?} unsupported, keeping current.", mode);
                return false;
            }

            let ok = SDL_SetGPUSwapchainParameters(
                self.device,
                window.raw(),
                SDL_GPUSwapchainComposition::SDL_GPU_SWAPCHAINCOMPOSITION_SDR,
                mode.0,
            );

            if !ok {
                debug!("Failed to set swapchain parameters: {}", sdl_last_error());
            }

            ok
        }
    }
}
