mod builder;
mod context;
mod scene;

use crossbeam_channel::{Sender, bounded};
use math::Size;
use renderer::GPU;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use traccia::{error, info, warn};
use wgpu::naga::FastHashMap;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    window::{WindowAttributes, WindowId},
};

// Re-exports
pub use crate::context::*;
pub use builder::{AppBuilder, WindowBuilder};
pub use common::{label, utils::Label};
pub use scene::Scene;

struct CustomFormatter;

impl traccia::Formatter for CustomFormatter {
    fn format(&self, record: &traccia::Record) -> String {
        format!(
            "{} {}",
            record.level.default_coloring().to_lowercase(),
            record.message
        )
    }
}

pub(crate) fn init_logging() {
    traccia::init_with_config(traccia::Config {
        level: if cfg!(debug_assertions) {
            traccia::LogLevel::Debug
        } else {
            traccia::LogLevel::Info
        },
        format: Some(Box::new(CustomFormatter)),
        ..Default::default()
    });
}

pub enum WindowCommand {
    Event(WindowEvent),
    Redraw,
    Close,
    MonitorsChanged(Vec<Monitor>),
}

struct WindowHandle {
    sender: Sender<WindowCommand>,
    handle: JoinHandle<()>,
}

struct PendingWindow {
    attributes: WindowAttributes,
    scenes: FastHashMap<Label, Box<dyn Scene>>,
}

pub struct App {
    gpu: Arc<GPU>,
    windows: FastHashMap<WindowId, WindowHandle>,
    pending_windows: Vec<PendingWindow>,
}

impl App {
    pub(crate) fn new() -> Self {
        let gpu = Arc::new(pollster::block_on(GPU::init()));
        let info = gpu.info();

        info!("backend: {}", info.backend);
        info!("device: {}", info.name);
        info!("device type: {:?}", info.device_type);
        info!("driver: {}", info.driver_info);

        Self {
            gpu,
            windows: FastHashMap::default(),
            pending_windows: Vec::new(),
        }
    }

    fn add_pending_window(
        &mut self,
        attributes: WindowAttributes,
        scenes: FastHashMap<Label, Box<dyn Scene>>,
    ) {
        self.pending_windows
            .push(PendingWindow { attributes, scenes });
    }

    fn spawn_window(
        &mut self,
        window: Arc<winit::window::Window>,
        scenes: FastHashMap<Label, Box<dyn Scene>>,
        recommended_fps: u32,
    ) {
        let (tx, rx) = bounded::<WindowCommand>(64);
        let gpu = Arc::clone(&self.gpu);
        let window_id = window.id();
        let window = Window::new(window);

        let handle = thread::spawn(move || {
            let mut ctx = Context::new(gpu, window, recommended_fps);
            let mut scenes = scenes;
            let mut active_scene = label!("initial");

            scenes
                .get_mut(&active_scene)
                .expect("cannot fail")
                .load(&mut ctx);

            // Kickstart the loop
            ctx.window.request_redraw();

            loop {
                match rx.recv() {
                    Ok(WindowCommand::Close) => return,

                    Ok(WindowCommand::Event(event)) => {
                        ctx.handle_event(event);
                    }

                    Ok(WindowCommand::MonitorsChanged(monitors)) => {
                        ctx.monitors.update(monitors);
                    }

                    Ok(WindowCommand::Redraw) => {
                        // Drain all pending events before rendering to avoid lag
                        while let Ok(cmd) = rx.try_recv() {
                            match cmd {
                                WindowCommand::Close => return,
                                WindowCommand::Event(event) => {
                                    ctx.handle_event(event);
                                }

                                WindowCommand::MonitorsChanged(monitors) => {
                                    ctx.monitors.update(monitors);
                                }
                                WindowCommand::Redraw => {
                                    // Skip redundant redraw commands
                                    break;
                                }
                            }
                        }

                        ctx.time.frame_start();
                        ctx.time.update();

                        while let Some(tick_start) = ctx.time.should_tick() {
                            if let Some(scene) = scenes.get_mut(&active_scene) {
                                scene.fixed_update(&mut ctx);
                            }

                            ctx.time.do_tick(tick_start);
                        }

                        if let Some(scene) = scenes.get_mut(&active_scene) {
                            scene.update(&mut ctx);
                            scene.render(&mut ctx);
                        }

                        match ctx.render.present(&ctx.gpu) {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::OutOfMemory) => return,
                            Err(e) => error!("Render error: {:?}", e),
                        }

                        ctx.time.frame_end();
                        ctx.input.flush();

                        let sleep_duration = ctx.time.until_next_frame();
                        if !sleep_duration.is_zero() {
                            spin_sleep::sleep(sleep_duration);
                        }

                        ctx.window.request_redraw();
                    }

                    Err(_) => return, // Channel closed
                }
            }
        });

        self.windows
            .insert(window_id, WindowHandle { sender: tx, handle });
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::new().expect("Failed to create event loop");

        // Don't poll because the main thread doesnt have the window loop, so just wait for events
        event_loop.set_control_flow(ControlFlow::Wait);
        event_loop.run_app(&mut self).expect("Failed to run app");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let pending = std::mem::take(&mut self.pending_windows);

        let recommended_fps = event_loop
            .available_monitors()
            .max_by(|a, b| {
                let a_hz = a.refresh_rate_millihertz().unwrap_or(60_000);
                let b_hz = b.refresh_rate_millihertz().unwrap_or(60_000);

                match a_hz.cmp(&b_hz) {
                    std::cmp::Ordering::Equal => {
                        // If refresh rates are equal, compare resolution (area)
                        let a_size: Size<u32> = a.size().into();
                        let b_size: Size<u32> = b.size().into();

                        a_size.area().cmp(&b_size.area())
                    }

                    other => other,
                }
            })
            .and_then(|m| {
                let size = m.size();
                let hz =
                    (m.refresh_rate_millihertz().unwrap_or(60_000) as f32 / 1000.0).round() as u32;

                info!(
                    "setting as recommended monitor: '{}' {}x{}@{}hz",
                    m.name().unwrap_or(String::from("unknown")),
                    size.width,
                    size.height,
                    hz,
                );

                Some(hz)
            })
            .unwrap_or(60);

        for window_config in pending {
            match event_loop.create_window(window_config.attributes) {
                Ok(window) => {
                    let window = Arc::new(window);

                    self.spawn_window(window, window_config.scenes, recommended_fps);
                }
                Err(e) => {
                    error!("Failed to create window: {}", e);
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(handle) = self.windows.remove(&window_id) {
                    let _ = handle.sender.send(WindowCommand::Close);
                    let _ = handle.handle.join();
                }

                if self.windows.is_empty() {
                    event_loop.exit();
                }
            }

            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(handle) = self.windows.get(&window_id) {
                    let monitors = Monitors::collect(event_loop);

                    warn!("Monitors changed. {} detected", monitors.len());

                    let _ = handle
                        .sender
                        .try_send(WindowCommand::MonitorsChanged(monitors));
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(handle) = self.windows.get(&window_id) {
                    if let Err(_) = handle.sender.try_send(WindowCommand::Redraw) {
                        // Redraw events can be safely dropped - next frame will render anyway
                    }
                }
            }

            _ => {
                if let Some(handle) = self.windows.get(&window_id) {
                    if let Err(_) = handle.sender.try_send(WindowCommand::Event(event)) {
                        warn!("Event dropped - channel full. Consider processing events faster.");
                    }
                }
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        for (_, handle) in self.windows.drain() {
            let _ = handle.sender.send(WindowCommand::Close);
            let _ = handle.handle.join();
        }
    }
}
