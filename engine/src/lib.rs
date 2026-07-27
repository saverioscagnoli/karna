mod builder;
mod event;
mod render;
mod scene;
mod window;

use std::mem;
use std::time::Instant;

use gpu::Gpu;
use logging::error;
use logging::fatal;
use logging::info;
use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::gpu::PresentMode;
use sdl3::gpu::SwapchainComposition;
use sdl3::pixels::Color;
use utils::FastHashMap;
use utils::SleepTimer;
use utils::WindowId;

use crate::event::SdlEvent;
use crate::event::SdlWindowEvent;
use crate::render::packet::FramePacket;
use crate::scene::World;
use crate::window::context::WindowContext;
use crate::window::state::WindowState;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::render::immediate::Draw;
pub use crate::render::retained::SceneRef;
pub use crate::scene::Scene;
pub use crate::window::Window;
pub use crate::window::context::ContextRef;
pub use crate::window::time::Time;

pub struct App {
    queued_windows: Vec<WindowBuilder>,

    windows: FastHashMap<WindowId, WindowState>,
    should_quit: bool,
    sleeper: SleepTimer,
    gpu: Gpu,
    video: VideoSubsystem,
    sdl: Sdl,
}

impl App {
    pub(crate) fn new() -> Self {
        let sdl = match sdl3::init() {
            Ok(sdl) => sdl,
            Err(e) => fatal!("Failed to initialize sdl: {}", e),
        };

        let sleeper = SleepTimer::calibrated();

        let video = match sdl.video() {
            Ok(v) => v,
            Err(e) => fatal!("Failed to initialized video subsystem: {}", e),
        };

        let gpu = match Gpu::init() {
            Ok(gpu) => gpu,
            Err(e) => fatal!("Failed to initialize gpu: {}", e),
        };

        gpu.log_shader_formats();
        gpu.log_info();

        Self {
            queued_windows: Vec::new(),
            windows: FastHashMap::default(),
            should_quit: false,
            sleeper,
            gpu,
            video,
            sdl,
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    fn spawn_window(&mut self, b: WindowBuilder) {
        let sdl_window = match self
            .video
            .window(&b.title, b.size.width, b.size.height)
            .build()
        {
            Ok(w) => w,
            Err(e) => {
                error!("Failed to create window: {}", e);
                return;
            }
        };

        if let Err(e) = self.gpu.claim_window(&sdl_window) {
            error!("The GPU failed to claim the window: {}", e);
            return;
        }

        let _ = self.gpu.device.set_swapchain_parameters(
            &sdl_window,
            PresentMode::Immediate,
            SwapchainComposition::default(),
        );

        let state = WindowState::new(
            WindowContext {
                window: Window::wrap(sdl_window),
                packet: FramePacket::default(),
            },
            World::new(b.scenes, b.active_scenes),
        );

        self.windows.insert(state.window.id(), state);
    }

    fn handle_event(&mut self, event: SdlEvent) {
        match event {
            SdlEvent::Window {
                window_id,
                win_event,
                ..
            } => {
                if let SdlWindowEvent::CloseRequested = win_event {
                    let Some(state) = self.windows.remove(&window_id) else {
                        error!("Received an event for a closed window.");
                        return;
                    };

                    self.gpu.release_window(&state.window.inner);
                    info!("Closing window '{}'", state.window.title());

                    if self.windows.is_empty() {
                        self.should_quit = true;
                    }

                    return;
                }

                let Some(state) = self.windows.get(&window_id) else {
                    error!("Received an event for a closed window.");
                    return;
                };

                match win_event {
                    _ => {}
                }
            }

            _ => {}
        }
    }

    pub fn run(mut self) {
        for b in mem::take(&mut self.queued_windows) {
            self.spawn_window(b);
        }

        let mut pump = match self.sdl.event_pump() {
            Ok(p) => p,
            Err(e) => fatal!("Failed to initialize SDL event pump: {}", e),
        };

        for state in self.windows.values_mut() {
            state.load();
        }

        while !self.should_quit {
            for event in pump.poll_iter() {
                self.handle_event(event);
            }

            let now = Instant::now();

            for state in self.windows.values_mut() {
                state.clock.advance(now);

                while state.clock.should_tick() {
                    state.tick();
                    state.clock.consume(now);
                }

                if !state.pacer.due(now) {
                    continue;
                }

                state.update();
                state.draw();

                let _ = self.gpu.clear(&state.window.inner, Color::CYAN);

                state.pacer.record(now);
                state.pacer.schedule(now);

                state.flush();
            }

            let deadline = self
                .windows
                .values()
                .map(|s| s.pacer.next_frame.min(s.clock.next_tick()))
                .min();

            match deadline {
                Some(d) => self.sleeper.sleep_until(d),
                None => self.should_quit = true,
            };
        }

        info!("All windows were closed. Exiting.");
    }
}
