mod builder;
mod context;
mod events;
mod lifecycle;
mod scene;

use std::mem;
use std::sync::Arc;
use std::thread;

pub use builder::AppBuilder;
pub use builder::WindowBuilder;
pub use context::ContextRef;
pub use context::ContextRefMut;
pub use context::Draw;
pub use context::Time;
pub use context::Window;
use logging::LogLevel;
use logging::error;
use logging::info;
use logging::trace;
use logging::warn;
use renderer::Renderer;
pub use scene::Scene;
pub use scene::SceneManager;
pub use scene::SceneMap;
use utils::FastHashMap;
use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowId;

use crate::events::EventHandler;
use crate::events::WindowHandle;
use crate::lifecycle::WindowLifecycle;

struct EngineLogTarget;

impl logging::target::Target for EngineLogTarget {
    fn write(&self, level: LogLevel, message: &str) -> Result<(), logging::LogError> {
        let mut logs = globals::logs::get().write();

        logs.push((level, message.to_string()));

        Ok(())
    }
}

fn init_logging() {
    logging::init(
        logging::Config::default().with_target(logging::TargetConfig {
            target: Box::new(EngineLogTarget),
            formatter: None,
        }),
    );
}

pub struct App {
    window_builders: Vec<WindowBuilder>,

    threads: FastHashMap<WindowId, WindowHandle>,
    events: EventHandler,
}

impl App {
    fn new() -> Self {
        Self {
            window_builders: Vec::new(),
            threads: FastHashMap::default(),
            events: EventHandler::new(),
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    fn init(&mut self) {
        gpu::init();
        init_logging();
        renderer::init();

        info!("App initialized successfully.");
    }

    /// Queues a window for spawning
    pub(crate) fn queue_window(&mut self, builder: WindowBuilder) {
        self.window_builders.push(builder);
    }

    fn spawn_window(&mut self, window: winit::window::Window, scenes: SceneMap) {
        let (window_tx, window_rx) = crossbeam_channel::unbounded::<WindowEvent>();

        let window_id = window.id();
        let winit_window = Arc::new(window);

        // Window surface must be created on the main thread on windows
        // because it sucks ass
        let (surface, config) = Renderer::create_surface(winit_window.clone());

        let window = Window::new(winit_window);

        let thread = thread::spawn(move || {
            let mut lifecycle = WindowLifecycle::new(window_rx, window, surface, config, scenes);

            lifecycle.game_loop();
        });

        let window_handle = WindowHandle::new(window_tx, thread);

        self.threads.insert(window_id, window_handle);
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");

        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self).expect("Failed to run app");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init();

        for b in mem::take(&mut self.window_builders) {
            match event_loop.create_window(b.attributes()) {
                Ok(w) => self.spawn_window(w, b.scenes),
                Err(e) => error!("Failed to create window: {}", e),
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            let Some(window) = self.threads.remove(&window_id) else {
                error!("Trying to close a non-existing window");
                return;
            };

            let WindowHandle { event_tx, thread } = window;

            // Send close event so the window thread's game loop exits.
            let _ = event_tx.send(event);

            // Drop the sender so the channel disconnects,
            // unblocking any recv() call on the window thread.
            drop(event_tx);

            // Join the window thread to ensure GPU resources are fully
            // cleaned up before we continue. Without this, the detached
            // thread may hold onto wgpu surfaces/resources while the
            // remaining windows try to use the shared device, causing stalls.
            if let Err(e) = thread.join() {
                error!("Window thread panicked: {:?}", e);
            }

            if self.threads.is_empty() {
                warn!("All windows were closed. Exiting.");
                event_loop.exit();
            }

            return;
        }

        let Some(window) = self.threads.get(&window_id) else {
            trace!("Received an event for a non-exisiting window");
            return;
        };

        // Propagate the event to the respective window
        if let Err(e) = window.event_tx.try_send(event) {
            warn!(
                "Event dropped: {} - Window channel full. Window id: {:?}",
                e, window_id
            );
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        // Stream the event to all windows
        let _ = self.events.device_tx.try_send(event);
    }
}
