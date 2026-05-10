mod builder;
mod context;
mod events;
mod init;
mod lifecycle;
mod scene;

use std::mem;
use std::sync::Arc;
use std::thread;

use assets::AssetServer;
pub use builder::AppBuilder;
pub use builder::WindowBuilder;
pub use context::ContextRef;
pub use context::ContextRefMut;
pub use context::Time;
pub use context::Window;
use info::SystemInfo;
use logging::Color;
use logging::Colorize;
use logging::ctx;
use logging::error;
use logging::info;
use logging::trace;
use logging::warn;
use renderer::Renderer;
pub use scene::Scene;
pub use scene::SceneManager;
pub use scene::SceneMap;
use utils::ByteSize;
use utils::FastHashMap;
use utils::Lazy;
use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
pub use winit::keyboard::KeyCode;
use winit::window::CustomCursor;
use winit::window::WindowId;

use crate::context::Monitors;
use crate::events::EventHandler;
use crate::events::MainCmd;
use crate::events::WindowHandle;
use crate::init::init_logging;
use crate::lifecycle::WindowLifecycle;

pub struct App {
    window_builders: Vec<WindowBuilder>,

    threads: FastHashMap<WindowId, WindowHandle>,
    /// Winit windows live on the winit thread. Keep them here so we can apply
    /// winit-thread-only operations (like creating/applying custom cursors).
    windows: FastHashMap<WindowId, Arc<winit::window::Window>>,

    events: EventHandler,
    asset_server: Lazy<AssetServer>,
    sysinfo: Lazy<SystemInfo>,
}

impl App {
    fn new() -> Self {
        Self {
            window_builders: Vec::new(),
            threads: FastHashMap::default(),
            windows: FastHashMap::default(),
            events: EventHandler::new(),
            asset_server: Lazy::new(),
            sysinfo: Lazy::new(),
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    fn init(&mut self) {
        init_logging();

        gpu::init();
        renderer::init();

        self.asset_server.set(AssetServer::new());
        self.sysinfo
            .set(SystemInfo::fetch(gpu::adapter().get_info()));

        info!("Cpu: {}", self.sysinfo.cpu_model);
        info!("Cpu cores: {}", self.sysinfo.cpu_cores);
        info!("Mem: {}", ByteSize::from_bytes(self.sysinfo.mem_total));
        info!("Gpu: {}", self.sysinfo.gpu_model);
        info!("Gpu driver: {}", self.sysinfo.graphics_driver);
        info!("Graphics backend: {}", self.sysinfo.graphics_backend);
        info!("App initialized successfully.");
    }

    /// Queues a window for spawning
    pub(crate) fn queue_window(&mut self, builder: WindowBuilder) {
        self.window_builders.push(builder);
    }

    fn spawn_window(
        &mut self,
        window: winit::window::Window,
        event_loop: &ActiveEventLoop,
        scenes: SceneMap,
    ) {
        let (window_tx, window_rx) = crossbeam_channel::unbounded::<WindowEvent>();

        let window_id = window.id();
        let winit_window = Arc::new(window);

        // Keep a handle on the winit thread for cursor/icon/etc operations.
        self.windows.insert(window_id, winit_window.clone());

        // Window surface must be created on the main thread on windows
        // because it sucks ass
        let (surface, config) = Renderer::create_surface(winit_window.clone());

        // Arc underneath so clone is the same instance
        let asset_server_clone = self.asset_server.clone();

        let window = Window::new(
            winit_window,
            self.asset_server.clone(),
            self.events.cmd_tx.clone(),
        );

        let mut monitors = Monitors::new(window.clone());

        monitors.update(Monitors::collect(event_loop));

        let thread = thread::spawn(move || {
            let _ctx = ctx!("window", window.title().color(Color::Magenta));

            let mut lifecycle = WindowLifecycle::new(
                window_rx,
                window,
                surface,
                config,
                monitors,
                scenes,
                asset_server_clone,
            );

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
                Ok(w) => self.spawn_window(w, event_loop, b.scenes),
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
        // Drain any pending app commands. These must execute on the winit thread.
        while let Ok(cmd) = self.events.cmd_rx.try_recv() {
            match cmd {
                MainCmd::SetCustomCursor {
                    window_id,
                    image,
                    hotspot_x,
                    hotspot_y,
                } => {
                    let Some(window) = self.windows.get(&window_id) else {
                        warn!("SetCustomCursor for unknown window_id={:?}", window_id);
                        continue;
                    };

                    let guard = self.asset_server.guard();
                    let img = guard.get_image(image);

                    let source = CustomCursor::from_rgba(
                        img.rgba.clone(),
                        img.size.width as u16,
                        img.size.height as u16,
                        hotspot_x,
                        hotspot_y,
                    )
                    .expect("failed to create custom cursor source from rgba");

                    let cursor = event_loop.create_custom_cursor(source);
                    window.set_cursor(cursor);
                }
            }
        }

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

            let _ = self.windows.remove(&window_id);

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
        //
        // IMPORTANT (Windows [wow!]): never block the winit event loop thread waiting for a window-thread
        // acknowledgement (e.g. on resize). If the window thread is stalled (GPU, surface, etc),
        // blocking here stops message pumping and the OS will mark the window as "Not Responding".
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
