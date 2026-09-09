#![no_std]

use nostd::log;
use nostd::mem::SdlAllocator;
use sdl3::SdlGuard;
use traccia::info;

#[global_allocator]
static ALLOC: SdlAllocator = SdlAllocator;

pub struct App {
    _sdl: SdlGuard,
}

impl App {
    pub fn new() -> Self {
        let Ok(_sdl) = SdlGuard::init() else {
            panic!("sdl failed to init");
        };

        log::capture();

        info!("SDL v{} initialized.", sdl3::linked_version());

        Self { _sdl }
    }
}
