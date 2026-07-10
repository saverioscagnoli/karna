mod builder;
mod context;
mod scene;
mod state;
mod time;
mod window;

use std::mem;
use std::thread;

use gpu::GpuState;
use gpu::WindowSurface;
use logging::error;
use logging::info;
use logging::warn;
use renderer::FramePacket;
use renderer::Renderer;
use utils::FastHashMap;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowId;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::context::ContextMut;
pub use crate::context::ContextRef;
pub use crate::context::Draw;
pub use crate::scene::Scene;
pub use crate::scene::SceneBuilder;
pub use crate::scene::SceneManager;
use crate::scene::Scenes;
use crate::state::WindowState;
pub use crate::time::Time;
pub use crate::window::Window;
use crate::window::WindowHandle;

pub struct App {
    enqueued_windows: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowHandle>,
    renderer: Renderer<WindowId>,
}

/// Private or crate-private implementations
impl App {
    fn new() -> Self {
        gpu::init(|shaders, d| {
            let src = include_str!("../../shaders/immediate-2d.wgsl");
            shaders.load("immediate-2d", src, d);
        });

        Self {
            enqueued_windows: Vec::new(),
            windows: FastHashMap::default(),
            renderer: Renderer::new(),
        }
    }

    pub(crate) fn request_window(&mut self, b: WindowBuilder) {
        self.enqueued_windows.push(b);
    }

    fn window_thread(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: Window,
        scenes: Scenes,
        active_scenes: Vec<String>,
    ) {
        let gpu = GpuState::get();

        // Initialize channels
        let (packet_tx, packet_rx) = crossbeam_channel::unbounded::<FramePacket>();
        let (event_tx, event_rx) = crossbeam_channel::unbounded::<WindowEvent>();

        let window_id = window.id();
        let surface = WindowSurface::create(gpu, window.winit_handle(), window.size());

        let thread = thread::spawn(move || {
            let state = WindowState::new(window, scenes, active_scenes, event_rx, packet_tx);

            state.start();
        });

        let handle = WindowHandle {
            thread,
            surface,
            event_tx,
            packet_rx,
        };

        self.windows.insert(window_id, handle);
    }
}

/// Public implementation
impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Failed to build event loop");

        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self).expect("Failed to run app")
    }
}

/// Winit implementation
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        for b in mem::take(&mut self.enqueued_windows) {
            let title = b.attrs.title.clone();

            match event_loop.create_window(b.attrs) {
                Ok(w) => {
                    // Wrap winit window
                    let window = Window::new(w);

                    info!("Created window '{}' {:?}", window.title(), window.size());

                    self.window_thread(event_loop, window, b.scenes, b.initial_active);
                }

                Err(e) => error!("Failed to spawn window '{}': {}", title, e),
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.windows.get_mut(&window_id) else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                let Some(window) = self.windows.remove(&window_id) else {
                    warn!(
                        "Received event but coulndn't find the window: {:?}",
                        window_id
                    );

                    return;
                };

                _ = window.event_tx.send(event);
                _ = window.thread.join();

                if self.windows.is_empty() {
                    info!("All windows were closed. Exiting.");
                    event_loop.exit();
                    return;
                }
            }

            WindowEvent::Resized(size) => {
                let gpu = GpuState::get();
                let size: math::Size<u32> = size.into();
                window.surface.resize(gpu, size);
            }

            WindowEvent::RedrawRequested => {
                if let Ok(packet) = window.packet_rx.try_recv() {
                    self.renderer
                        .present(window_id, &mut window.surface, packet);
                }
            }

            event => {
                if let Err(e) = window.event_tx.send(event) {
                    error!("Failed to send window event: {}", e);
                }
            }
        }
    }
}
