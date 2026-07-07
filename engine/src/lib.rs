mod builder;
mod context;
pub mod input;
mod logs;
mod monitors;
mod scene;
mod shared;
mod time;
mod window;
mod window_state;

use std::mem;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use gpu::GpuState;
use imgui::ActiveImgui;
use logging::error;
use logging::info;
use logging::warn;
use renderer::Renderer;
use utils::FastHashMap;
use utils::Lazy;
use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::WindowEvent;
use winit::event::WindowEvent::CloseRequested;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::window::WindowId;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::context::ContextRef;
pub use crate::context::ContextRefMut;
pub use crate::input::Input;
pub use crate::input::KeyCode;
pub use crate::input::MouseButton;
use crate::logs::LOGS;
pub use crate::logs::init_logging;
use crate::monitors::Monitor;
use crate::monitors::Monitors;
pub use crate::scene::Scene;
pub use crate::scene::SceneManager;
use crate::scene::Scenes;
use crate::shared::SharedResources;
pub use crate::window::Window;
use crate::window::WindowHandle;
use crate::window::WinitWindow;
use crate::window_state::WindowState;

pub enum AppEvent {
    WindowEvent(WindowEvent),
    DeviceEvent(Arc<DeviceEvent>),
    QueryMonitors(Vec<Monitor>),
}

enum UserEvent {
    Shutdown,
}

pub struct App {
    enqueued_windows: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowHandle>,
    shared: Lazy<SharedResources>,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            windows: FastHashMap::default(),
            enqueued_windows: Vec::new(),
            shared: Lazy::empty(),
        }
    }

    fn init(&mut self) {
        gpu::init(|shader_store, d| {
            let immediate_2d_src = include_str!("../../shaders/immediate-2d.wgsl");
            let immediate_2d_circles_src = include_str!("../../shaders/immediate-2d-circles.wgsl");

            shader_store.load("immediate-2d", immediate_2d_src, d);
            shader_store.load("immediate-2d-circles", immediate_2d_circles_src, d);

            info!("Built-in shaders loaded");
        });

        let gpu = GpuState::get();

        info!("GPU initialization complete");

        let info = gpu.device.adapter_info();

        info!(
            "GPU: {} ({:?}, {}, driver {})",
            info.name, info.device_type, info.backend, info.driver_info
        );

        self.shared.set(SharedResources::new());

        info!("App initialization complete")
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub(crate) fn request_window(&mut self, b: WindowBuilder) {
        self.enqueued_windows.push(b)
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::<UserEvent>::with_user_event()
            .build()
            .expect("Failed to create event loop");

        let proxy = event_loop.create_proxy();

        ctrlc::set_handler(move || {
            _ = proxy.send_event(UserEvent::Shutdown);
            info!("Shutdown requested");
        })
        .expect("Failed to set Ctrl+C handler");

        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self).expect("Failed to run app")
    }
}

impl App {
    fn spawn_window_thread(
        &mut self,
        event_loop: &ActiveEventLoop,
        winit_window: Arc<WinitWindow>,
        scenes: Scenes,
        active_scenes: Vec<String>,
    ) {
        info!("Creating window  '{}'", winit_window.title());

        let (tx, rx) = mpsc::channel::<AppEvent>();
        let window_for_thread = winit_window.clone();
        let window_id = winit_window.id();

        // We must query the monitor before the
        // scene(s) are loaded, so that they are
        // available during that time
        // otherwise, in the inital scene loading, the
        // vector will be empty
        let monitors = Monitors::collect(event_loop);

        // On Windows, the surface must be created on the winit thread,
        // hence why the renderer creation is split.
        // If we were to create the renderer in the thread::spawn it would crash
        let (surface, config) = Renderer::_create_surface(winit_window.clone());

        let logs = LOGS.clone();
        let shared = self.shared.clone();
        let SharedResources { assets, imgui } = shared;

        let thread_handle = thread::spawn(move || {
            let window = Window::new(window_for_thread);

            imgui.guard().register_window(window_id);

            info!("Asset server initialization complete");

            let mut active_imgui = ActiveImgui::new(&imgui, window_id);
            let renderer =
                Renderer::_from_surface(surface, config, assets._guard(), &mut active_imgui, logs);

            drop(active_imgui);

            info!("Renderer initialization complete");

            WindowState::new(
                rx,
                window,
                assets,
                renderer,
                scenes,
                active_scenes,
                monitors,
                imgui,
            )
            .start_loop();
        });

        let window_handle = WindowHandle {
            sender: tx,
            thread: thread_handle,
            window: winit_window,
        };

        self.windows.insert(window_id, window_handle);
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init();

        for b in mem::take(&mut self.enqueued_windows) {
            match event_loop.create_window(b.attrs) {
                Ok(w) => {
                    self.spawn_window_thread(event_loop, Arc::new(w), b.scenes, b.initial_active)
                }
                Err(e) => error!("Failed to spawn window: {}", e),
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::Shutdown => {
                for (_, window) in self.windows.drain() {
                    _ = window.sender.send(AppEvent::WindowEvent(CloseRequested));
                    _ = window.thread.join();
                }

                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                let Some(window) = self.windows.remove(&window_id) else {
                    error!("CloseRequested on a non existing window");
                    return;
                };

                _ = window.sender.send(AppEvent::WindowEvent(event));
                _ = window.thread.join();

                let mut imgui = self.shared.imgui.guard();

                imgui.unregister_window(window.window.id());

                if self.windows.is_empty() {
                    info!("All windows were closed. Exiting.");
                    event_loop.exit();
                }
            }

            WindowEvent::ScaleFactorChanged { .. } => {
                let Some(window) = self.windows.get(&window_id) else {
                    return;
                };

                let monitors = Monitors::collect(event_loop);

                info!("Monitor(s) detected: {}", monitors.len());

                if let Err(e) = window.sender.send(AppEvent::QueryMonitors(monitors)) {
                    warn!(e:err;"Dropped event '{:?}'", event);
                }
            }

            event => {
                let Some(window) = self.windows.get(&window_id) else {
                    warn!("Event received on a non existing window (expected when closing one)");
                    return;
                };

                if let Err(e) = window.sender.send(AppEvent::WindowEvent(event)) {
                    warn!(e:err;"Dropped event'");
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let event = Arc::new(event);

        for w in self.windows.values() {
            if let Err(e) = w.sender.send(AppEvent::DeviceEvent(event.clone())) {
                error!(e:err;  "Failed to broadcast device event");
            }
        }
    }
}
