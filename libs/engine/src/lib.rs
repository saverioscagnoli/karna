#![no_std]

use core::time::Duration;

use nostd::alloc::vec::Vec;
use nostd::collections::HashMap;
use nostd::log;
use nostd::mem::SdlAllocator;
use nostd::time::Instant;
use nostd::time::sleep_precise;
use sdl3::SdlGuard;
use sdl3::events::SdlEvent;
use sdl3::gpu::Device;
use sdl3::window::Window;
use sdl3::window::WindowId;
use traccia::info;

#[global_allocator]
static ALLOC: SdlAllocator = SdlAllocator;

pub struct App {
    windows: HashMap<WindowId, Window>,
    device: Device,
    should_quit: bool,
    _sdl: SdlGuard,
}

impl App {
    pub fn new() -> Self {
        let Ok(_sdl) = SdlGuard::init() else {
            panic!("sdl failed to init");
        };

        log::capture();
        info!("SDL v{} initialized.", sdl3::linked_version());

        let Ok(device) = sdl3::gpu::Device::init() else {
            panic!("failed to init gpu device");
        };

        Self {
            windows: HashMap::default(),
            device,
            should_quit: false,
            _sdl,
        }
    }

    fn spawn_window(&mut self, title: &str, size: math::Size<u32>) {
        let window = self
            .device
            .create_window(title, size)
            .expect("Failed to create window");

        self.windows.insert(window.id(), window);
    }

    fn close_window(&mut self, id: WindowId) {
        let Some(entry) = self.windows.remove(&id) else {
            return;
        };

        if self.windows.is_empty() {
            self.should_quit = true;
        }
    }

    fn quit(&mut self) {
        for id in self.windows.keys().copied().collect::<Vec<_>>() {
            self.close_window(id);
        }
    }

    pub fn run(mut self) {
        self.spawn_window("hello", math::size!(1280, 720));

        const PERIOD: Duration = Duration::from_nanos(1_000_000_000 / 60);

        let mut last = Instant::now();
        let mut next = Instant::now();

        while !self.should_quit {
            let now = Instant::now();
            let dt = (now - last).as_secs_f32();
            last = now;

            for event in sdl3::events::poll() {
                match event {
                    SdlEvent::Quit => self.quit(),
                    _ => {}
                }
            }

            next += PERIOD;
            let now = Instant::now();

            if next > now {
                sleep_precise(next - now);
            } else if now - next > PERIOD {
                next = now;
            }
        }
    }
}
