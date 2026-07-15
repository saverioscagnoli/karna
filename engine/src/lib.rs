mod builder;
mod context;
pub mod input;
mod mixer;
mod resources;
mod scene;
mod state;
mod time;
mod window;

#[cfg(feature = "net")]
pub mod net;

use std::mem;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::thread;

use assets::AssetServer;
use assets::Image;
use gpu::GpuState;
use gpu::WindowSurface;
use imgui::SharedImgui;
use logging::debug;
use logging::error;
use logging::info;
use logging::trace;
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
    SpawnWindow(WindowBuilder),
    SetCustomCursor(Arc<WinitWindow>, Handle<Image>, math::Vector2<u16>),
}

fn load_builtin_shaders(shaders: &mut gpu::ShaderStore, d: &gpu::wgpu::Device) {
    let src = include_str!("../../shaders/immediate-2d.wgsl");
    let src1 = include_str!("../../shaders/immediate-2d-circles.wgsl");
    let src2 = include_str!("../../shaders/mesh-3d.wgsl");

    shaders.load("immediate-2d", src, d);
    shaders.load("immediate-2d-circles", src1, d);
    shaders.load("mesh-3d", src2, d);

    debug!("Built-in shaders loaded.");
}

/// Everything that can only exist *after* the gpu singleton is initialized.
///
/// On native this is built synchronously at the start of [`App::run`]; on the
/// web it is built inside the spawned future once `gpu::init_async` has
/// completed, because gpu initialization must be awaited there.
struct AppShared {
    pipelines: Arc<gpu::PipelineCache>,
    layouts: Arc<Layouts>,
    assets: AssetServer,
    imgui: SharedImgui,
}

impl AppShared {
    /// Panics if the gpu singleton has not been initialized yet.
    fn new() -> Self {
        let gpu = GpuState::get();

        debug!("Gpu initialization complete.");

        let info = gpu.device.adapter_info();

        info!(
            "GPU: {} ({:?}, {}, driver {})",
            info.name, info.device_type, info.backend, info.driver_info
        );

        let assets = AssetServer::new();
        let layouts = {
            let r = assets.read();
            Arc::new(Layouts::new(r.atlas_bgl()))
        };

        let imgui = SharedImgui::new();

        debug!("Initialized shared imgui context");

        Self {
            pipelines: Arc::new(gpu::PipelineCache::new()),
            layouts,
            assets,
            imgui,
        }
    }
}

pub struct App {
    // ---- State ----
    enqueued_windows: Vec<WindowBuilder>,
    windows: FastHashMap<WindowId, WindowHandle>,
    focused: Option<WindowId>,

    // ---- User event ----
    proxy: Lazy<EventLoopProxy<UserEvent>>,

    // ---- Shared (refcounted), set once the gpu is ready ----
    shared: Lazy<AppShared>,
}

/// Private or crate-private implementations
impl App {
    fn new() -> Self {
        Self {
            enqueued_windows: Vec::new(),
            windows: FastHashMap::default(),
            focused: None,
            proxy: Lazy::empty(),
            shared: Lazy::empty(),
        }
    }

    pub(crate) fn request_window(&mut self, b: WindowBuilder) {
        self.enqueued_windows.push(b);
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop, b: WindowBuilder) {
        // On the web a "window" is a canvas; have winit append it to the
        // document body so it actually shows up. Also remember the size the
        // builder asked for — the browser may still report 0x0 at creation.
        #[cfg(target_arch = "wasm32")]
        let (attrs, requested_size) = {
            use winit::platform::web::WindowAttributesExtWebSys;
            let requested_size = b.attrs.inner_size;
            (b.attrs.with_append(true), requested_size)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let attrs = b.attrs;

        let title = attrs.title.clone();

        match event_loop.create_window(attrs) {
            Ok(w) => {
                // Wrap winit window
                let window = Window::new(w, self.shared.assets.reader(), self.proxy.clone());

                info!("Created window '{}' {:?}", window.title(), window.size());

                #[cfg(not(target_arch = "wasm32"))]
                self.window_thread(event_loop, window, b.scenes, b.initial_active);

                #[cfg(target_arch = "wasm32")]
                self.spawn_window(window, b.scenes, b.initial_active, requested_size);
            }

            Err(e) => error!("Failed to spawn window '{}': {}", title, e),
        }
    }

    /// Native: each window gets its own thread running the blocking
    /// render/update loop; events are forwarded through a channel.
    #[cfg(not(target_arch = "wasm32"))]
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

        let pipelines = self.shared.pipelines.clone();
        let layouts = self.shared.layouts.clone();
        let window_arc = window.clone();
        let assets = self.shared.assets.clone();
        let imgui = self.shared.imgui.clone();

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

    /// Web: no threads — the window state lives on the main thread and its
    /// frame loop is driven by `RedrawRequested` (i.e. requestAnimationFrame).
    /// Events still go through the same channel so `WindowState` is
    /// identical on both targets.
    #[cfg(target_arch = "wasm32")]
    fn spawn_window(
        &mut self,
        window: Window,
        scenes: Scenes,
        active_scenes: Vec<String>,
        requested_size: Option<winit::dpi::Size>,
    ) {
        let gpu = GpuState::get();

        let (event_tx, event_rx) = crossbeam_channel::unbounded::<AppEvent>();

        let window_id = window.id();

        // The browser reports the canvas size asynchronously (through a
        // ResizeObserver), so it can still be 0x0 here. Fall back to the
        // size the builder asked for and let the state pick it up as a
        // regular resize on its first frame.
        let mut size = window.size();

        if (size.width == 0 || size.height == 0)
            && let Some(requested) = requested_size
        {
            let physical = requested.to_physical::<u32>(1.0);
            size = math::Size::new(physical.width, physical.height);

            _ = event_tx.send(AppEvent::Window(WindowEvent::Resized(physical)));
        }

        let surface = WindowSurface::create(gpu, window.winit(), size);

        let renderer = Renderer::new(
            self.shared.pipelines.clone(),
            self.shared.layouts.clone(),
            self.shared.assets.reader(),
        );

        let state = WindowState::new(
            window.clone(),
            renderer,
            surface,
            scenes,
            active_scenes,
            self.shared.assets.clone(),
            self.shared.imgui.clone(),
            event_rx,
        );

        // Kick off the rAF-driven frame loop
        window.request_redraw();

        let handle = WindowHandle {
            event_tx,
            state,
            window,
        };

        self.windows.insert(window_id, handle);
    }
}

/// Public implementation
impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(mut self) {
        gpu::init(load_builtin_shaders);
        self.shared.set(AppShared::new());

        let event_loop = EventLoop::with_user_event()
            .build()
            .expect("Failed to build event loop");

        self.proxy.set(event_loop.create_proxy());

        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self).expect("Failed to run app")
    }

    /// Web entry point: gpu init must be awaited, so everything runs inside
    /// a spawned future. Returns immediately; the event loop keeps the app
    /// alive from browser callbacks.
    #[cfg(target_arch = "wasm32")]
    pub fn run(mut self) {
        use winit::platform::web::EventLoopExtWebSys;

        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        wasm_bindgen_futures::spawn_local(async move {
            gpu::init_async(load_builtin_shaders).await;
            self.shared.set(AppShared::new());

            let event_loop = EventLoop::with_user_event()
                .build()
                .expect("Failed to build event loop");

            self.proxy.set(event_loop.create_proxy());

            event_loop.set_control_flow(ControlFlow::Wait);
            event_loop.spawn_app(self);
        });
    }
}

/// Winit implementation
impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        for b in mem::take(&mut self.enqueued_windows) {
            self.create_window(event_loop, b);
        }

        info!("App initialization complete.");
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        trace!("Received user event {:?}", event);

        match event {
            UserEvent::SpawnWindow(b) => self.create_window(event_loop, b),

            UserEvent::SetCustomCursor(window, image, hotspot) => {
                let assets = self.shared.assets.read();
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
        match event {
            WindowEvent::CloseRequested => {
                let Some(handle) = self.windows.remove(&window_id) else {
                    warn!(
                        "Received event but couldn't find the window: {:?}",
                        window_id
                    );

                    return;
                };

                _ = handle.event_tx.send(AppEvent::Window(event));

                #[cfg(not(target_arch = "wasm32"))]
                {
                    _ = handle.thread.join();
                }

                self.shared.imgui.unregister_window(window_id);

                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }

            WindowEvent::Focused(is) => {
                if is {
                    self.focused = Some(window_id);
                    debug!("Window {:?} gained focus.", window_id);
                }
            }

            // On the web the frame loop is driven from here: each redraw
            // runs one frame and schedules the next one (rAF-style).
            #[cfg(target_arch = "wasm32")]
            WindowEvent::RedrawRequested => {
                let Some(handle) = self.windows.get_mut(&window_id) else {
                    return;
                };

                if handle.state.frame_once() {
                    handle.window.request_redraw();
                } else {
                    self.windows.remove(&window_id);
                    self.shared.imgui.unregister_window(window_id);

                    if self.windows.is_empty() {
                        event_loop.exit();
                    }
                }
            }

            event => {
                let Some(handle) = self.windows.get_mut(&window_id) else {
                    return;
                };

                if let Err(e) = handle.event_tx.send(AppEvent::Window(event)) {
                    error!("Failed to send window event: {}", e);

                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        let window = self.windows.remove(&window_id).unwrap();

                        _ = window.thread.join();

                        if self.windows.is_empty() {
                            info!("All windows were closed. Exiting.");
                            event_loop.exit();
                        }
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
