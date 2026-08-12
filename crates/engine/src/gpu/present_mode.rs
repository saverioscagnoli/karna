use logging::debug;
use logging::warn;
use sdl3::SDL_GPUPresentMode;
use sdl3::SDL_GPUSwapchainComposition;
use sdl3::SDL_SetGPUSwapchainParameters;
use sdl3::SDL_WindowSupportsGPUPresentMode;

use crate::err::SDL_LastError;
use crate::gpu::Gpu;
use crate::window::platform::PlatformWindow;

#[derive(Debug, Clone, Copy)]
pub enum PresentMode {
    VSync,
    Immediate,
    Mailbox,
}

impl PresentMode {
    pub fn sdl(self) -> SDL_GPUPresentMode {
        match self {
            Self::VSync => SDL_GPUPresentMode::SDL_GPU_PRESENTMODE_VSYNC,
            Self::Immediate => SDL_GPUPresentMode::SDL_GPU_PRESENTMODE_IMMEDIATE,
            Self::Mailbox => SDL_GPUPresentMode::SDL_GPU_PRESENTMODE_MAILBOX,
        }
    }
}

impl Gpu {
    pub fn set_present_mode(&self, window: &PlatformWindow, mode: PresentMode) {
        unsafe {
            if !SDL_WindowSupportsGPUPresentMode(self.device, window.raw(), mode.sdl()) {
                warn!("Present mode {:?} is not supported on this device.", mode);
                return;
            }

            let success = SDL_SetGPUSwapchainParameters(
                self.device,
                window.raw(),
                SDL_GPUSwapchainComposition::SDL_GPU_SWAPCHAINCOMPOSITION_SDR,
                mode.sdl(),
            );

            if !success {
                warn!(
                    "Failed to set present mode to {:?}: {}",
                    mode,
                    SDL_LastError()
                );

                return;
            }

            debug!("Present mode set to {:?}", mode);
        }
    }
}
