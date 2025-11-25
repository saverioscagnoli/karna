pub mod context;
pub mod input;
pub mod scene;
mod time;

use crate::{context::Context, scene::Scene};
use nalgebra::Vector2;
use std::sync::Arc;
use wgpu::naga::FastHashMap;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub struct App {
    scenes: FastHashMap<String, Box<dyn Scene>>,
    current_scene: String,
    initial_size: Vector2<u32>,
    window: Option<Arc<Window>>,
    context: Option<Context>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            scenes: FastHashMap::default(),
            current_scene: String::new(),
            initial_size: Vector2::new(800, 600),
            window: None,
            context: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = PhysicalSize::new(self.initial_size.x, self.initial_size.y);
        let attributes = Window::default_attributes()
            .with_inner_size(size)
            .with_resizable(false);

        let window = event_loop.create_window(attributes).expect(":(");
        let window = Arc::new(window);

        let mut context = Context::new(window.clone());

        match self.scenes.get_mut(&self.current_scene) {
            Some(scene) => scene.load(&mut context),
            None => panic!("You have to set the initial scene!"),
        }

        self.window = Some(window.clone());
        self.context = Some(context);

        window.request_redraw();
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(ctx) = &mut self.context else {
            return;
        };

        let Some(scene) = self.scenes.get_mut(&self.current_scene) else {
            return;
        };

        ctx.time.start_frame();

        // Update the time struct first
        // with delta time
        ctx.time
            .update((ctx.time.this_frame - ctx.time.last_frame).as_secs_f32());

        // Then fixed update (tick), then unrestricted update
        while let Some(update_start) = ctx.time.should_tick() {
            scene.fixed_update(ctx);
            ctx.time.tick(update_start);
        }

        // Give the opportunity for things that should
        // be updated in `fixed_update`, then update
        scene.update(ctx);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(ctx) = &mut self.context else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                ctx.render.resize([size.width, size.height].into());
            }

            WindowEvent::KeyboardInput { event, .. } => match event.physical_key {
                PhysicalKey::Code(code) => {
                    if let KeyCode::Escape = code {
                        event_loop.exit();
                    }

                    if event.state.is_pressed() {
                        if !event.repeat {
                            ctx.input.keys_pressed.insert(code);
                        }

                        ctx.input.keys_held.insert(code);
                    } else {
                        ctx.input.keys_held.remove(&code);
                    }
                }
                _ => {}
            },

            WindowEvent::RedrawRequested => {
                let Some(window) = &mut self.window else {
                    return;
                };

                if let Some(scene) = self.scenes.get(&self.current_scene) {
                    scene.render(ctx);
                }

                ctx.render.end_frame();
                ctx.time.end_frame();
                ctx.input.flush();

                window.request_redraw();

                spin_sleep::sleep(ctx.time.until_next_frame());
            }

            _ => {}
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(mut self, size: Vector2<u32>) -> Self {
        self.initial_size = size;
        self
    }

    pub fn with_scene<L: AsRef<str>>(mut self, label: L, scene: Box<dyn Scene>) -> Self {
        self.scenes.insert(label.as_ref().to_string(), scene);
        self
    }

    pub fn with_current_scene<L: AsRef<str>>(mut self, label: L) -> Self {
        self.current_scene = label.as_ref().to_string();
        self
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::with_user_event().build().expect(":(");

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect(":(");
    }
}
