mod builder;
mod context;
mod scene;

use crate::context::WinitWindow;
use crossbeam_channel::{Receiver, Sender};
use ctor::ctor;
use math::Size;
use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};
use traccia::{error, info, warn};
use wgpu::naga::FastHashMap;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};

// Re-exports
pub use builder::{AppBuilder, WindowBuilder};
pub use context::{Context, Monitor, Monitors, Time, Window, input};
pub use scene::Scene;
pub use utils::{
    label,
    map::{Label, LabelMap},
};

fn init_logging() {
    traccia::init_with_config(traccia::Config {
        level: if cfg!(debug_assertions) {
            traccia::LogLevel::Debug
        } else {
            traccia::LogLevel::Info
        },
        format: Some(Box::new(
            traccia::FormatterBuilder::new()
                .with_span_position(traccia::SpanPosition::AfterLevel)
                .build(|record, span| {
                    if span.is_empty() {
                        format!(
                            "{} {}",
                            record.level.default_coloring().to_lowercase(),
                            record.message
                        )
                    } else {
                        format!(
                            "{} {} {}",
                            record.level.default_coloring().to_lowercase(),
                            span,
                            record.message
                        )
                    }
                }),
        )),
        ..Default::default()
    });
}

#[ctor]
/// The gpu will be initialized before the app starts, so that things like
/// font cache, geometry cache can be used during scene creation.
///
/// Maybe in the future i can implement a scenebuilder with the
/// gpu available, or make the user explicitly call something like karna::init();
fn init() {
    gpu::init();
    init_logging();
}

enum WindowMessage {
    Close,
    MonitorsChanged(Vec<Monitor>),
    StartFrame,
    WinitEvent(WindowEvent),
}

struct WindowHandle {
    sender: Sender<WindowMessage>,
    thread: JoinHandle<()>,
}

pub struct App {
    windows: FastHashMap<WindowId, WindowHandle>,
    window_builders: Vec<WindowBuilder>,
}

/// Internal
/// App creation
/// Game loop
impl App {
    pub(crate) fn new() -> Self {
        Self {
            windows: FastHashMap::default(),
            window_builders: Vec::new(),
        }
    }

    pub(crate) fn add_window_builder(&mut self, builder: WindowBuilder) {
        self.window_builders.push(builder);
    }

    fn spawn_window_thread(
        &mut self,
        label: String,
        window: WinitWindow,
        scenes: LabelMap<Box<dyn Scene>>,
    ) {
        let (tx, rx) = crossbeam_channel::bounded::<WindowMessage>(64);
        let window_id = window.id();
        let window = Window::new(label, window);

        let handle = thread::spawn(move || {
            let _span = traccia::span!("window", "label" => window.label());
            Self::window_loop(window, rx, scenes);
        });

        let window_handle = WindowHandle {
            sender: tx,
            thread: handle,
        };

        self.windows.insert(window_id, window_handle);
    }

    fn window_loop(
        window: Window,
        rx: Receiver<WindowMessage>,
        mut scenes: LabelMap<Box<dyn Scene>>,
    ) {
        let mut context = Context::new(window);
        let active_scene = label!("initial");

        scenes
            .get_mut(&active_scene)
            .expect("Cannot fail")
            .load(&mut context);

        // Kickstart
        context.window.request_redraw();

        loop {
            match rx.recv() {
                Ok(WindowMessage::Close) => {
                    info!("Close requested");
                    return;
                }

                Ok(WindowMessage::MonitorsChanged(monitors)) => {
                    context.monitors.update(monitors);
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
                                context.monitors.update(monitors);
                            }

                            WindowMessage::WinitEvent(event) => {
                                if let WindowEvent::Resized(size) = event {
                                    let size: Size<u32> = size.into();

                                    if let Some(scene) = scenes.get_mut(&active_scene) {
                                        scene.on_resize(size, &mut context);
                                    }
                                }

                                context.handle_event(event);
                            }
                        }
                    }

                    context.time.frame_start();
                    context.time.update();

                    while let Some(tick_start) = context.time.next_tick() {
                        if let Some(scene) = scenes.get_mut(&active_scene) {
                            scene.fixed_update(&mut context);
                        }

                        context.time.do_tick(tick_start);
                    }

                    if let Some(scene) = scenes.get_mut(&active_scene) {
                        scene.update(&mut context);
                        scene.render(&mut context);
                    }

                    match context.render.present() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::OutOfMemory) => return,
                        Err(e) => error!("Render error: {:?}", e),
                    }

                    context.time.frame_end();
                    context.input.flush();
                    context.time.wait_for_next_frame();
                    context.window.request_redraw();
                }

                Ok(WindowMessage::WinitEvent(event)) => {
                    if let WindowEvent::Resized(size) = event {
                        let size: Size<u32> = size.into();

                        if let Some(scene) = scenes.get_mut(&active_scene) {
                            scene.on_resize(size, &mut context);
                        }
                    }

                    context.handle_event(event);
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
        let windows = std::mem::take(&mut self.window_builders);

        for builder in windows {
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
