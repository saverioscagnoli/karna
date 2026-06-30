mod builder;
mod context;
pub mod input;
mod scene;
mod time;
mod window;
mod window_state;

use std::mem;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;

use gpu::GpuState;
use logging::debug;
use logging::error;
use logging::info;
use logging::warn;
use renderer::Renderer;
use utils::FastHashMap;
use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::DeviceId;
use winit::event::WindowEvent;
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
pub use crate::scene::Scene;
pub use crate::scene::SceneManager;
use crate::scene::Scenes;
pub use crate::window::Window;
use crate::window::WindowHandle;
use crate::window::WinitWindow;
use crate::window_state::WindowState;

pub enum AppEvent {
    WindowEvent(WindowEvent),
    DeviceEvent(Arc<DeviceEvent>),
}

pub struct App {
    enqueued_windows: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowHandle>,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            windows: FastHashMap::default(),
            enqueued_windows: Vec::new(),
        }
    }

    fn init(&self) {
        gpu::init(|shader_store, d| {
            let immediate_2d_src = include_str!("../../shaders/immediate-2d.wgsl");
            shader_store.load("immediate-2d", immediate_2d_src, d);

            info!("Built-in shaders loaded.");
        });

        let gpu = GpuState::get();
        let info = gpu.device.adapter_info();

        info!("Gpu: {}", info.name);
        info!("Gpu type: {:?}", info.device_type);
        info!("Backend: {}", info.backend);
        info!("Driver: {}", info.driver_info);
        info!("App initialization complete")
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub(crate) fn request_window(&mut self, b: WindowBuilder) {
        self.enqueued_windows.push(b)
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");

        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self).expect("Failed to run app")
    }
}

impl App {
    fn spawn_window_thread(
        &mut self,
        winit_window: Arc<WinitWindow>,
        scenes: Scenes,
        active_scenes: Vec<String>,
    ) {
        let (tx, rx) = mpsc::channel::<AppEvent>();
        let window_id = winit_window.id();

        // On Windows, the surface must be created on the winit thread,
        // hence why the renderer creation is split.
        // If we were to create the renderer in the thread::spawn it would crash
        let (surface, config) = Renderer::create_surface(winit_window.clone());

        let thread_handle = thread::spawn(move || {
            let window = Window::new(winit_window);
            let renderer = Renderer::from_surface(surface, config);

            WindowState::new(window, renderer, scenes, active_scenes, rx).start_loop();
        });

        let window_handle = WindowHandle {
            sender: tx,
            thread: thread_handle,
        };

        self.windows.insert(window_id, window_handle);
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init();

        for b in mem::take(&mut self.enqueued_windows) {
            match event_loop.create_window(b.attrs) {
                Ok(w) => self.spawn_window_thread(Arc::new(w), b.scenes, b.initial_active),
                Err(e) => error!("Failed to spawn window: {}", e),
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
            let Some(window) = self.windows.remove(&window_id) else {
                error!("CloseRequested on a non existing window");
                return;
            };

            _ = window.sender.send(AppEvent::WindowEvent(event));
            _ = window.thread.join();

            if self.windows.is_empty() {
                info!("All windows were closed. Exiting.");
                event_loop.exit();
            }

            return;
        }

        let Some(window) = self.windows.get(&window_id) else {
            debug!("Event received on a non existing window");
            return;
        };

        if let Err(e) = window.sender.send(AppEvent::WindowEvent(event)) {
            warn!(e:err; "Event dropped, channel full.");
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
