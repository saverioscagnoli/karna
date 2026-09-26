use sdl3_sys::SDL_GPU_PRESENTMODE_IMMEDIATE;
use sdl3_sys::SDL_GPU_PRESENTMODE_MAILBOX;
use sdl3_sys::SDL_GPU_PRESENTMODE_VSYNC;
use sdl3_sys::SDL_GPUPresentMode;

#[derive(Debug, Clone, Copy)]
pub enum PresentMode {
    Vsync,
    Mailbox,
    Immediate,
}

impl PresentMode {
    pub fn to_sdl(&self) -> SDL_GPUPresentMode {
        match self {
            Self::Vsync => SDL_GPU_PRESENTMODE_VSYNC,
            Self::Mailbox => SDL_GPU_PRESENTMODE_MAILBOX,
            Self::Immediate => SDL_GPU_PRESENTMODE_IMMEDIATE,
        }
    }
}
