mod context;
mod scene;
mod time;
mod window;

use crate::input::KeyState;
use crate::scene::SceneManager;
use math::Size;
use spin_sleep::SpinSleeper;
use std::sync::Arc;
use std::time::Instant;
use traccia::{Color as TColor, Colorize, LogLevel, Style, fatal, info, warn};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
#[cfg(not(feature = "imgui"))]
use winit::event;
use winit::event::{Event, WindowEvent};
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

        let mut context = match Context::new(window.clone()) {
            Ok(ctx) => ctx,
            Err(e) => {
                fatal!("Failed to create context: {}", e);
                event_loop.exit();
                return;
            }
        };

        // let info = context.render.info();

        // info!("backend: {}", info.backend);
        // info!("device: {}", info.name);
        // info!("device type: {:?}", info.device_type);
        // info!("driver: {}", info.driver_info);

        if let Some(scene) = self.scenes.current_mut() {
            scene.load(&mut context);
        }

        self.context = Some(context);
    }

    fn user_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        #[cfg(feature = "imgui")] mut event: Context,
        #[cfg(not(feature = "imgui"))] event: Context,
    ) {
        // #[cfg(feature = "imgui")]
        //  event.render.imgui.handle_event(&Event::UserEvent(()));

        self.context = Some(event);
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let Some(context) = &mut self.context else {
            warn!("Received a device event before initializing context");
            return;
        };

        // #[cfg(feature = "imgui")]
        // context
        //     .render
        //     .imgui
        //     .handle_event(&Event::DeviceEvent { device_id, event });
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(context) = &mut self.context else {
            return;
        };

        context.time.this_frame = Instant::now();

        let dt = context.time.this_frame - context.time.last_frame;

        #[cfg(feature = "imgui")]
        {
            // context.render.imgui.handle_event(&Event::AboutToWait);
            // context.render.imgui.update_dt(dt);
        }

        context.time.last_frame = context.time.this_frame;
        context.time.update_accum += dt;

        context.time.tick(dt);

        if let Some(scene) = self.scenes.current_mut() {
            scene.update(context);
        }

        while context.time.update_accum >= context.time.ups_step {
            let update_start = Instant::now();

            if let Some(scene) = self.scenes.current_mut() {
                scene.fixed_update(context);
            }

            context.time.update_accum -= context.time.ups_step;
            context.time.tick_fixed(update_start);
        }

        context.input.flush();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(context) = &mut self.context else {
            return;
        };

        match &event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                // context.render.resize((*size).into());
            }

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => match key_event.physical_key {
                PhysicalKey::Code(code) => {
                    let state = context
                        .input
                        .keys
                        .entry(code)
                        .or_insert(KeyState::default());

                    if key_event.state.is_pressed() {
                        if !key_event.repeat {
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
                    .entry(*button)
                    .or_insert(KeyState::default());

                if state.is_pressed() {
                    s.held = true;
                    s.pressed = true;
                } else {
                    s.held = false;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                context.input.mouse_pos = (*position).into();
            }

            WindowEvent::RedrawRequested => {
                if let Some(scene) = self.scenes.current_mut() {
                    scene.render(context);
                }

                // context.render.present();
                context.window.request_redraw();
                context.time.frame_time = Instant::now() - context.time.this_frame;

                // if !context.render.vsync() {
                //     let sleep_duration = context
                //         .time
                //         .fps_step
                //         .saturating_sub(context.time.frame_time);

                //     self.sleeper.sleep(sleep_duration);
                // }
            }

            _ => {}
        }

        // #[cfg(feature = "imgui")]
        // context
        //     .render
        //     .imgui
        //     .handle_event(&Event::WindowEvent { window_id, event });
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            context: None,
            window_size: Size::new(800, 600),
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
