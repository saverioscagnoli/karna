mod window;

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

use crate::window::SdlEvent;
use crate::window::SdlWindowEvent;
use crate::window::context::WindowContext;
use crate::window::state::WindowState;

pub use crate::window::Window;
use crate::window::time::Time;

pub struct App {
    windows: FastHashMap<WindowId, WindowState>,
    should_quit: bool,
    sleeper: SleepTimer,
    gpu: Gpu,
    video: VideoSubsystem,
    sdl: Sdl,
}

impl App {
    pub fn new() -> Self {
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
            windows: FastHashMap::default(),
            should_quit: false,
            sleeper,
            gpu,
            video,
            sdl,
        }
    }

    fn spawn_window(&mut self, title: &str, width: u32, height: u32) {
        let sdl_window = match self.video.window(title, width, height).build() {
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

        let state = WindowState {
            context: WindowContext {
                window: Window::wrap(sdl_window),
                time: Time::default(),
            },
        };

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
        self.spawn_window("demo", 800, 600);

        let mut pump = match self.sdl.event_pump() {
            Ok(p) => p,
            Err(e) => fatal!("Failed to initialize SDL event pump: {}", e),
        };

        while !self.should_quit {
            for event in pump.poll_iter() {
                self.handle_event(event);
            }

            let now = Instant::now();

            for state in self.windows.values_mut() {
                if !state.time.frame_due(now) {
                    continue;
                }

                state.time.advance();

                while state.time.should_tick() {
                    let start = Instant::now();
                    state.time.consume(start);
                }

                let _ = self.gpu.clear(&state.window.inner, Color::BLACK);
                state.time.schedule_next_frame();
            }

            match self.windows.values().map(|s| s.time.next_frame()).min() {
                Some(deadline) => self.sleeper.sleep_until(deadline),
                None => break,
            }
        }

        info!("All windows were closed. Exiting.");
    }
}
