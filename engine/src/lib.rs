mod context;
mod scene;
mod time;
mod window;

use crate::input::KeyState;
use crate::scene::SceneManager;
use math::Size;
use spin_sleep::SpinSleeper;
use std::sync::Arc;
use std::time::{Duration, Instant};
use traccia::{Color as TColor, Colorize, LogLevel, Style, info};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::{Window, WindowId};

// Re-exports
pub mod input;
pub use crate::context::Context;
pub use scene::Scene;

struct LogFormatter;

impl traccia::Formatter for LogFormatter {
    fn format(&self, record: &traccia::Record) -> String {
        let date = chrono::Local::now().format("%m/%d %H:%M:%S").to_string();
        format!(
            "{} [{}] {} {}",
            date.color(TColor::Cyan).dim(),
            record.target.dim(),
            record.level.default_coloring().to_lowercase(),
            record.message
        )
    }
}

pub struct App {
    context: Option<Context>,
    window_size: Size<u32>,
    acc: f32,
    sleeper: SpinSleeper,
    scenes: SceneManager,
}

impl ApplicationHandler<Context> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size: PhysicalSize<u32> = self.window_size.into();
        let attributes = Window::default_attributes()
            .with_inner_size(size)
            .with_resizable(false);

        let window = event_loop
            .create_window(attributes)
            .expect("Failed to create window");

        let window = Arc::new(window);

        let mut context = Context::new(window);
        let info = context.render.info();

        info!("backend: {}", info.backend);
        info!("device type: {:?}", info.device_type);
        info!("driver: {}", info.driver_info);
        info!("card: {}", info.name);

        if let Some(scene) = self.scenes.current_mut() {
            scene.load(&mut context);
        }

        self.context = Some(context);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Context) {
        self.context = Some(event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(context) = &mut self.context else {
            return;
        };

        context.time.t1 = Instant::now();
        let dt = context
            .time
            .t1
            .duration_since(context.time.t0)
            .as_secs_f32();

        context.time.t0 = context.time.t1;
        self.acc += dt;

        context.time.update(dt);

        if let Some(scene) = self.scenes.current_mut() {
            scene.update(context);
        }

        while self.acc >= context.time.ups_step {
            self.acc -= context.time.ups_step
        }

        context.input.flush();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(context) = &mut self.context else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                context.render._resize(size.into());
            }

            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(code) => {
                    let state = context
                        .input
                        .keys
                        .entry(code)
                        .or_insert(KeyState::default());

                    if event.state.is_pressed() {
                        if !event.repeat {
                            state.pressed = true;
                        }

                        state.held = true;
                    } else {
                        state.held = false;
                    }
                }
                PhysicalKey::Unidentified(_) => {}
            },

            WindowEvent::MouseInput { button, state, .. } => {
                let s = context
                    .input
                    .mouse
                    .entry(button)
                    .or_insert(KeyState::default());

                if state.is_pressed() {
                    s.held = true;
                    s.pressed = true;
                } else {
                    s.held = false;
                }
            }

            WindowEvent::CursorMoved { position, .. } => context.input.mouse_pos = position.into(),

            WindowEvent::RedrawRequested => {
                if let Some(scene) = self.scenes.current_mut() {
                    scene.render(context);
                }

                context.render._present();

                if !context.render.vsync() {
                    context.time.t2 = context.time.t1.elapsed().as_secs_f32();
                    let sleep_time = context.time.fps_step - context.time.t2;

                    if sleep_time > 0.0 {
                        self.sleeper.sleep(Duration::from_secs_f32(sleep_time));
                    }
                }

                context.window.request_redraw();
            }

            _ => {}
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            context: None,
            window_size: Size::new(800, 600),
            acc: 0.0,
            sleeper: SpinSleeper::default(),
            scenes: SceneManager::new(),
        }
    }

    pub fn with_size<S: Into<Size<u32>>>(mut self, size: S) -> Self {
        self.window_size = size.into();
        self
    }

    pub fn with_scene<T: AsRef<str>, S: Scene + 'static>(mut self, scene_id: T, scene: S) -> Self {
        if self.scenes.current == "" {
            self.scenes.current = scene_id.as_ref().to_string()
        }

        self.scenes.add(scene_id, Box::new(scene));

        self
    }

    pub fn run(mut self) -> Result<(), String> {
        traccia::init_with_config(traccia::Config {
            level: if cfg!(debug_assertions) {
                LogLevel::Debug
            } else {
                LogLevel::Info
            },
            format: Some(Box::new(LogFormatter)),
            ..Default::default()
        });

        let event_loop = EventLoop::with_user_event()
            .build()
            .expect("Failed to create event loop");

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop
            .run_app(&mut self)
            .expect("Failed to run application");

        Ok(())
    }
}
