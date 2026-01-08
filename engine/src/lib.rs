mod builder;
mod scene;
mod state;

use crate::{
    scene::SceneManager,
    state::{EngineState, WinitWindow, states::GlobalStates, sysinfo::SystemInfo},
};
use assets::AssetServer;
use crossbeam_channel::{Receiver, Sender};
use globals::{TrackingAllocator, profiling};
use logging::{LogError, LogLevel, error, info, warn};
use math::Size;
use renderer::Renderer;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use utils::{FastHashMap, Lazy};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

// Re-exports
pub use builder::{AppBuilder, WindowBuilder};
pub use renderer::Draw;
pub use scene::Scene;
pub use state::{Context, Monitor, Monitors, RenderContext, Time, Window, input};
pub use utils::{Label, label};

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

struct EngineLogs;

impl logging::target::Target for EngineLogs {
    fn write(&self, level: LogLevel, message: &str) -> Result<(), LogError> {
        let logs = globals::logs::get();
        let mut lock = logs.write().map_err(|_| LogError::PoisonError)?;

        lock.push((level, message.to_string()));

        Ok(())
    }
}

enum WindowMessage {
    Close,
    MonitorsChanged(Vec<Monitor>),
    StartFrame,
    WinitEvent(WindowEvent),
    DeviceEvent(DeviceEvent),
}

struct WindowHandle {
    sender: Sender<WindowMessage>,
    thread: JoinHandle<()>,
}

#[derive(Clone)]
struct Arcs {
    globals: Arc<GlobalStates>,
    info: Arc<SystemInfo>,
}

pub struct App {
    windows: FastHashMap<WindowId, WindowHandle>,
    window_builders: Vec<WindowBuilder>,
    arcs: Lazy<Arcs>,
}

/// Internal
/// App creation
/// Game loop
impl App {
    pub(crate) fn new() -> Self {
        Self {
            windows: FastHashMap::default(),
            window_builders: Vec::new(),
            arcs: Lazy::new(),
        }
    }

    pub(crate) fn init(&mut self) {
        logging::init(
            logging::Config::default().with_target(logging::TargetConfig {
                target: Box::new(EngineLogs),
                formatter: None,
            }),
        );

        gpu::init();

        let info = Arc::new(SystemInfo::new());

        info!("Cpu: {} ({})", info.cpu_model(), info.cpu_cores());
        info!(
            "Total Memory: {:.2} GB",
            info.mem_total() as f64 / 1024.0 / 1024.0 / 1024.0
        );
        info!("Gpu: {}", info.gpu_model());
        info!("Gpu Type: {:?}", info.gpu_type());
        info!("Graphics Backend: {}", info.gpu_backend());
        info!("Graphics Driver: {}", info.gpu_driver());

        let globals = Arc::new(GlobalStates::new());

        self.arcs.set(Arcs { globals, info });
    }

    pub(crate) fn add_window_builder(&mut self, builder: WindowBuilder) {
        self.window_builders.push(builder);
    }

    fn spawn_window_thread(
        &mut self,
        label: String,
        window: WinitWindow,
        scenes: FastHashMap<Label, Box<dyn Scene>>,
    ) {
        let (tx, rx) = crossbeam_channel::unbounded::<WindowMessage>();
        let window_id = window.id();
        let window = Window::new(label, window);
        let label = window.label().to_string();
        let arcs = self.arcs.clone();

        // Must spawn the renderer on the main thread because windows sucks ass
        let assets = AssetServer::new();
        let renderer = Renderer::new(window.inner().clone(), &assets);

        let handle = thread::spawn(move || {
            let _ctx = logging::ctx!("window", label);
            Self::window_loop(scenes, window, renderer, assets, arcs, rx);
        });

        let window_handle = WindowHandle {
            sender: tx,
            thread: handle,
        };

        self.windows.insert(window_id, window_handle);
    }

    fn window_loop(
        scenes: FastHashMap<Label, Box<dyn Scene>>,
        window: Window,
        renderer: Renderer,
        assets: AssetServer,
        arcs: Arcs,
        rx: Receiver<WindowMessage>,
    ) {
        let mut state = EngineState::new(window, renderer, assets, arcs);
        let mut scenes = SceneManager::new(scenes);

        scenes.current().load(&mut state.as_context());

        // Kickstart
        state.window.request_redraw();

        loop {
            match rx.recv() {
                Ok(WindowMessage::Close) => {
                    info!("Close requested");
                    return;
                }

                Ok(WindowMessage::MonitorsChanged(monitors)) => {
                    state.monitors.update(monitors);
                }

                Ok(WindowMessage::StartFrame) => {
                    // Drain all events before rendering
                    while let Ok(message) = rx.try_recv() {
                        match message {
                            WindowMessage::Close => {
                                info!("Close requested");
                                return;
                            }

                            WindowMessage::StartFrame => {
                                // Skip
                                break;
                            }

                            WindowMessage::MonitorsChanged(monitors) => {
                                state.monitors.update(monitors);
                            }

                            WindowMessage::WinitEvent(event) => {
                                if let WindowEvent::Resized(size) = event {
                                    let size: Size<u32> = size.into();

                                    scenes.current().on_resize(size, &mut state.as_context());
                                }

                                state.handle_event(event);
                            }

                            WindowMessage::DeviceEvent(event) => {
                                state.handle_device_event(event);
                            }
                        }
                    }

                    // FRAME START
                    profiling::reset_frame();

                    state.time.frame_start();
                    state.time.update();

                    while let Some(tick_start) = state.time.next_tick() {
                        scenes.current().fixed_update(&mut state.as_context());
                        state.time.do_tick(tick_start);
                    }

                    scenes.current().update(&mut state.as_context());

                    let (render_context, mut draw) = state.as_render_context();

                    scenes.current().render(&render_context, &mut draw);

                    state.render.present(&state.assets);

                    state.time.frame_end();
                    state.input.flush();
                    state.time.wait_for_next_frame();
                    state.window.request_redraw();

                    // Check for pending scene changes
                    if let Some(new_scene) = state.scenes.changed_to() {
                        info!(
                            "Changing from scene '{:?}' to '{:?}'",
                            scenes.current_label(),
                            new_scene
                        );

                        scenes.switch_to(new_scene, &mut state.as_context());
                    }

                    state.profiling = profiling::get_stats();
                }

                Ok(WindowMessage::WinitEvent(event)) => {
                    if let WindowEvent::Resized(size) = event {
                        let size: Size<u32> = size.into();

                        scenes.current().on_resize(size, &mut state.as_context());
                    }

                    state.handle_event(event);
                }

                Ok(WindowMessage::DeviceEvent(event)) => {
                    state.handle_device_event(event);
                }

                Err(_) => return,
            }
        }
    }
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::new()
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");

        // Don't poll because the main thread doesnt have the window loop, so just wait for events
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self).expect("Failed to run app");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init();

        for builder in std::mem::take(&mut self.window_builders) {
            match event_loop.create_window(builder.attributes) {
                Ok(window) => {
                    self.spawn_window_thread(builder.label, Arc::new(window), builder.scenes);
                }

                Err(e) => {
                    error!(
                        "Failed to spawn window with label: '{}': {}",
                        builder.label, e
                    );
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: DeviceEvent,
    ) {
        for window in self.windows.values() {
            let _ = window
                .sender
                .try_send(WindowMessage::DeviceEvent(event.clone()));
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
                if let Some(window) = self.windows.remove(&window_id) {
                    let _ = window.sender.send(WindowMessage::Close);
                    let _ = window.thread.join();
                }

                if self.windows.is_empty() {
                    warn!("All windows were closed. Exiting");
                    event_loop.exit();
                }
            }

            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(window) = self.windows.get(&window_id) {
                    let monitors = Monitors::collect(event_loop);

                    warn!("Monitors changed");
                    warn!("{} monitors detected", monitors.len());

                    let _ = window
                        .sender
                        .try_send(WindowMessage::MonitorsChanged(monitors));
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(window) = self.windows.get(&window_id) {
                    if let Err(_) = window.sender.try_send(WindowMessage::StartFrame) {
                        // Redraw events can be safely dropped - next frame will render anyway
                    }
                }
            }

            _ => {
                if let Some(window) = self.windows.get(&window_id) {
                    if let Err(_) = window.sender.try_send(WindowMessage::WinitEvent(event)) {
                        warn!("Event dropped: channel full.");
                    }
                }
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        for (_, window) in self.windows.drain() {
            let _ = window.sender.send(WindowMessage::Close);
            let _ = window.thread.join();
        }
    }
}
