mod context;
mod scene;

use common::{label, utils::Label};
use math::Size;
use std::sync::Arc;
use wgpu::naga::FastHashMap;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowAttributes,
};

// Re-exports
pub use crate::{context::Context, scene::Scene};

pub struct App {
    initial_size: Size<u32>,
    context: Option<Context>,
    scenes: FastHashMap<Label, Box<dyn Scene>>,
    current_scene: Option<Label>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            initial_size: Size::new(800, 600),
            context: None,
            scenes: FastHashMap::default(),
            current_scene: None,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_scene<L: AsRef<str>>(mut self, label: L, scene: Box<dyn Scene>) -> Self {
        self.scenes.insert(label!(label.as_ref()), scene);
        self
    }

    pub fn with_initial_scene<L: AsRef<str>>(mut self, label: L, scene: Box<dyn Scene>) -> Self {
        let label = label.as_ref();

        self.scenes.insert(label!(label), scene);
        self.current_scene = Some(label!(label));
        self
    }

    pub fn add_scene<L: AsRef<str>>(&mut self, label: L, scene: Box<dyn Scene>) {
        self.scenes.insert(label!(label.as_ref()), scene);
    }

    pub fn set_scene<L: AsRef<str>>(&mut self, label: L) {
        let label = label!(label.as_ref());

        if self.scenes.get(&label).is_some() {
            self.current_scene = Some(label)
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::with_user_event().build().expect(":(");

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut self).expect(":(");
    }
}

impl ApplicationHandler<Context> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_inner_size(PhysicalSize::from(self.initial_size))
            .with_resizable(false);

        let window = event_loop
            .create_window(attributes)
            .expect("Failed to create winow");

        let window = Arc::new(window);
        let mut context = Context::new(window.clone());

        let label = match self.current_scene {
            Some(l) => l,
            None => {
                event_loop.exit();
                return;
            }
        };

        if let Some(scene) = self.scenes.get_mut(&label) {
            scene.load(&mut context);
        }

        self.context = Some(context);

        window.request_redraw();
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(ctx) = &mut self.context else { return };
        let Some(scene) = self.scenes.get_mut(&self.current_scene.unwrap()) else {
            return;
        };

        ctx.time.frame_start();

        // First, update delta time, timers, etc.
        ctx.time.update();

        // Then, fixed tick until the accumulator allows it
        while let Some(update_start) = ctx.time.should_tick() {
            scene.fixed_update(ctx);
            ctx.time.do_tick(update_start);
        }

        // Then, update the scene with an unrestricted update
        // Give the opportunity  for things that should be updated
        // with a fixed time to be updated first, like physics,
        // then update as fast as the fps allow
        scene.update(ctx);

        ctx.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(ctx) = &mut self.context else { return };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                ctx.render.resize(size.into());
            }

            WindowEvent::RedrawRequested => {
                if let Some(scene) = self.scenes.get_mut(&self.current_scene.unwrap()) {
                    scene.render(ctx);
                }

                ctx.render.present();
                ctx.time.frame_end();

                spin_sleep::sleep(ctx.time.until_next_frame());
            }

            _ => {}
        }
    }
}
