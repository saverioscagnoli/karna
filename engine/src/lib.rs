mod builder;
mod context;
pub mod input;
mod mixer;
mod scene;
mod state;
mod time;
mod window;

use std::mem;
use std::sync::Arc;
use std::thread;

use assets::AssetServer;
use assets::Image;
use gpu::GpuState;
use gpu::WindowSurface;
use imgui::SharedImgui;
use logging::debug;
use logging::error;
use logging::info;
use logging::warn;
use renderer::Layouts;
use renderer::Renderer;
use utils::FastHashMap;
use utils::Handle;
use utils::Lazy;
use winit::application::ApplicationHandler;
use winit::event::DeviceEvent;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;
use winit::window::CustomCursor;
use winit::window::WindowId;

pub use crate::builder::AppBuilder;
pub use crate::builder::WindowBuilder;
pub use crate::context::ContextMut;
pub use crate::context::ContextRef;
pub use crate::context::Draw;
pub use crate::context::SceneHandle;
pub use crate::scene::Scene;
pub use crate::scene::SceneBuilder;
pub use crate::scene::SceneManager;
use crate::scene::Scenes;
use crate::state::WindowState;
pub use crate::time::Time;
pub use crate::window::Window;
use crate::window::WindowHandle;
use crate::window::WinitWindow;

#[derive(Debug)]
pub enum AppEvent {
    Window(WindowEvent),
    Device(DeviceEvent),
}

#[derive(Debug)]
pub enum UserEvent {
    SetCustomCursor(Arc<WinitWindow>, Handle<Image>, math::Vector2<u16>),
}

pub struct App {
    // ---- State ----
    enqueued_windows: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowHandle>,
    focused: Option<WindowId>,

    // ---- User event ----
    proxy: Lazy<EventLoopProxy<UserEvent>>,

    // ---- Shared (refcounted) ----
    pipelines: Arc<gpu::PipelineCache>,
    layouts: Arc<Layouts>,
    assets: AssetServer,
    imgui: SharedImgui,
}

/// Private or crate-private implementations
impl App {
    fn new() -> Self {
        gpu::init(|shaders, d| {
            let src = include_str!("../../shaders/immediate-2d.wgsl");
            let src1 = include_str!("../../shaders/immediate-2d-circles.wgsl");
            let src2 = include_str!("../../shaders/mesh-3d.wgsl");

            shaders.load("immediate-2d", src, d);
            shaders.load("immediate-2d-circles", src1, d);
            shaders.load("mesh-3d", src2, d);

            debug!("Built-in shaders loaded.");
        });

        let gpu = GpuState::get();

        debug!("Gpu initialization complete.");

        let info = gpu.device.adapter_info();

        info!(
            "GPU: {} ({:?}, {}, driver {})",
            info.name, info.device_type, info.backend, info.driver_info
        );

        let assets = AssetServer::new();
        let r = assets.read();
        let layouts = Arc::new(Layouts::new(r.atlas_bgl()));

        drop(r);

        let imgui = SharedImgui::new();

        debug!("Initialized shared imgui context");

        Self {
            enqueued_windows: Vec::new(),
            windows: FastHashMap::default(),
            focused: None,
            proxy: Lazy::empty(),
            pipelines: Arc::new(gpu::PipelineCache::new()),
            layouts,
            assets,
            imgui,
        }
    }

    pub(crate) fn request_window(&mut self, b: WindowBuilder) {
        self.enqueued_windows.push(b);
    }

    fn window_thread(
        &mut self,
        _event_loop: &ActiveEventLoop,
        window: Window,
        scenes: Scenes,
        active_scenes: Vec<String>,
    ) {
        let gpu = GpuState::get();

        // Initialize channels
        let (event_tx, event_rx) = crossbeam_channel::unbounded::<AppEvent>();

        let window_id = window.id();
        let surface = WindowSurface::create(gpu, window.winit(), window.size());

        let pipelines = self.pipelines.clone();
        let layouts = self.layouts.clone();
        let window_arc = window.clone();
        let assets = self.assets.clone();
        let imgui = self.imgui.clone();

        let thread = thread::spawn(move || {
            let renderer = Renderer::new(pipelines, layouts, assets.reader());
            let state = WindowState::new(
                window,
                renderer,
                surface,
                scenes,
                active_scenes,
                assets,
                imgui,
                event_rx,
            );

            state.start();
        });

        let handle = WindowHandle {
            thread,
            event_tx,
            window: window_arc,
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
        let event_loop = EventLoop::with_user_event()
            .build()
            .expect("Failed to build event loop");

        self.proxy.set(event_loop.create_proxy());

        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self).expect("Failed to run app")
    }
}

/// Winit implementation
impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        for b in mem::take(&mut self.enqueued_windows) {
            let title = b.attrs.title.clone();

            match event_loop.create_window(b.attrs) {
                Ok(w) => {
                    // Wrap winit window
                    let window = Window::new(w, self.proxy.clone());

                    info!("Created window '{}' {:?}", window.title(), window.size());

                    self.window_thread(event_loop, window, b.scenes, b.initial_active);
                }

                Err(e) => error!("Failed to spawn window '{}': {}", title, e),
            }
        }

        info!("App initialization complete.");
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::SetCustomCursor(window, image, hotspot) => {
                let assets = self.assets.read();
                let image = assets.get_image(image);

                let source = CustomCursor::from_rgba(
                    image.data.clone(),
                    image.size.width as u16,
                    image.size.height as u16,
                    hotspot.x,
                    hotspot.y,
                )
                .expect("Failed to set cursor");

                let cursor = event_loop.create_custom_cursor(source);
                window.set_cursor(cursor);

                info!(
                    "Set custom cursor {:?} for window '{}'",
                    image.size,
                    window.title()
                );
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

                _ = window.event_tx.send(AppEvent::Window(event));
                _ = window.thread.join();

                self.imgui.unregister_window(window_id);

                if self.windows.is_empty() {
                    event_loop.exit();
                    return;
                }
            }

            WindowEvent::Focused(is) => {
                if is {
                    self.focused = Some(window_id);
                    debug!("Window {:?} gained focus.", window_id);
                }
            }

            event => {
                if let Err(e) = window.event_tx.send(AppEvent::Window(event)) {
                    error!("Failed to send window event: {}", e);

                    let window = self.windows.remove(&window_id).unwrap();

                    _ = window.thread.join();

                    if self.windows.is_empty() {
                        info!("All windows were closed. Exiting.");
                        event_loop.exit();
                        return;
                    }
                }
            }
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: winit::event::DeviceId, event: DeviceEvent) {
        if let Some(id) = self.focused
            && let Some(window) = self.windows.get(&id)
        {
            let _ = window.event_tx.send(AppEvent::Device(event));
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        info!("All windows were closed. Exiting.");
    }
}
