mod builder;
mod event;
mod logs;
mod scene;
mod window;

pub mod render;

use std::mem;
use std::sync::mpsc;
use std::thread;

use gpu::GpuState;
use gpu::WindowSurface;
use logging::debug;
use logging::error;
use logging::info;
use logging::warn;
use sdl3::event::Event;
use sdl3::event::WindowEvent;
use utils::FastHashMap;

use crate::render::Renderer;
use crate::window::MainThreadRequest;
use crate::window::Window;
use crate::window::WindowHandle;
use crate::window::WindowRequest;
use crate::window::context::FramePacket;
use crate::window::context::WindowContext;
use crate::window::input::Input;
use crate::window::state::WindowState;
use crate::window::time::Time;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::logs::init_logging;
pub use crate::render::Draw;
pub use crate::scene::Scene;
pub use crate::scene::SceneManager;
pub use crate::window::context::Context;

pub struct App {
    windows: FastHashMap<u32, WindowHandle>,
    events: sdl3::EventSubsystem,
    video: sdl3::VideoSubsystem,
    sdl: sdl3::Sdl,
    queued_windows: Vec<WindowBuilder>,
}

impl App {
    fn new() -> Self {
        let sdl = sdl3::init().expect("Failed to init sdl3");
        let video = sdl.video().expect("Failed to init video subsystem");
        let events = sdl.event().expect("Failed to init event subsystem");

        events.register_custom_event::<WindowRequest>().unwrap();

        let gpu = GpuState::get();
        debug!("GPU initialized.");

        info!("Video driver: {:?}", video.current_video_driver());

        let info = gpu.device.adapter_info();
        info!(
            "GPU: {} ({:?}, {}, driver {})",
            info.name, info.device_type, info.backend, info.driver_info
        );

        Self {
            windows: FastHashMap::default(),
            events,
            video,
            sdl,
            queued_windows: Vec::new(),
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    pub(crate) fn queue_window(&mut self, b: WindowBuilder) {
        self.queued_windows.push(b);
    }

    fn spawn_window(&mut self, b: WindowBuilder) {
        let sdl_builder = self.video.window(&b.title, b.size.width, b.size.height);
        let window = b.build_sdl(sdl_builder);

        let gpu = GpuState::get();

        let surface = WindowSurface::create(&gpu, &window);

        let window_id = window.id();
        let window_title = window.title().to_owned();
        let window_size = window.size().into();

        // Channels
        let (event_tx, event_rx) = mpsc::channel::<Event>();
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>();
        let proxy = self.events.event_sender();

        let thread = thread::spawn(move || {
            WindowState {
                context: WindowContext {
                    window: Window::new(window_id, window_title, window_size, proxy),
                    input: Input::new(),
                    time: Time::new(),
                    scenes: SceneManager::new(),
                    renderer: Renderer::new(surface),
                },
                scenes: b.scenes,
                active_scenes: b.initial_active,
                events: event_rx,
                shutdown: shutdown_rx,
                packet: FramePacket::default(),
                pending_input_timestamp: None,
            }
            .run_loop();
        });

        self.windows.insert(
            window_id,
            WindowHandle {
                window,
                event_sender: event_tx,
                shutdown: shutdown_tx,
                thread,
            },
        );
    }

    fn close_window(&mut self, window_id: u32) {
        let Some(handle) = self.windows.remove(&window_id) else {
            warn!("Trying to close a window that doesnt exist anymore");
            return;
        };

        let _ = handle.shutdown.send(());
        let _ = handle.thread.join();
    }

    pub fn run(mut self) {
        if self.queued_windows.is_empty() {
            warn!("No window was requested. Exiting.");
            return;
        }

        for b in mem::take(&mut self.queued_windows) {
            self.spawn_window(b);
        }

        let mut pump = self.sdl.event_pump().expect("Failed to get event pump");

        'main: loop {
            let event = pump.wait_event();

            if let Some(req) = event.as_user_event_type::<WindowRequest>() {
                match req.request {
                    MainThreadRequest::SetWindowTitle(t) => {
                        if let Some(handle) = self.windows.get_mut(&req.window_id) {
                            let _ = handle.window.set_title(&t);
                            debug!("Title set to '{}' for window {}", t, req.window_id);
                        }
                    }

                    MainThreadRequest::SetWindowSize(size) => {
                        if let Some(handle) = self.windows.get_mut(&req.window_id) {
                            let _ = handle.window.set_size(size.width, size.height);
                            debug!("Size set to {:?} for window {}", size, req.window_id);
                        }
                    }
                }

                continue 'main;
            }

            match event {
                Event::Quit { .. } => break 'main,
                Event::Window {
                    timestamp,
                    window_id: w_id,
                    win_event: w_event,
                } => match w_event {
                    WindowEvent::CloseRequested => {
                        self.close_window(w_id);

                        if self.windows.is_empty() {
                            info!("All windows were closed. Exiting.");
                            break 'main;
                        }
                    }

                    event => {
                        let Some(handle) = self.windows.get(&w_id) else {
                            warn!("Received event for a window that doesnt exist anymore");
                            return;
                        };

                        if let Err(e) = handle.event_sender.send(Event::Window {
                            timestamp,
                            window_id: w_id,
                            win_event: event,
                        }) {
                            error!("Failed to send event to window: {}", e);
                            return;
                        }
                    }
                },

                Event::KeyDown { window_id, .. }
                | Event::KeyUp { window_id, .. }
                | Event::MouseMotion { window_id, .. }
                | Event::MouseButtonDown { window_id, .. }
                | Event::MouseButtonUp { window_id, .. }
                | Event::MouseWheel { window_id, .. }
                | Event::TextInput { window_id, .. } => {
                    if let Some(handle) = self.windows.get(&window_id) {
                        let _ = handle.event_sender.send(event);
                    }
                }

                _ => {}
            }
        }
    }
}
