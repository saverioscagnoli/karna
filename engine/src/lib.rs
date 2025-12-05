pub mod context;
pub mod scene;

use crate::{context::Context, scene::Scene};
use common::{label, utils::Label};
use math::Size;
use renderer::{Renderer, SurfaceState};
use std::{intrinsics::ctlz, sync::Arc};
use traccia::fatal;
use wgpu::{ContextBlasTriangleGeometry, naga::FastHashMap};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowAttributes},
};

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

fn init_logging() {
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

pub struct App {
    context: Context,
    initial_size: Size<u32>,
    scenes: FastHashMap<Label, Box<dyn Scene>>,
    current_scene: Label,
}

impl App {
    pub fn new() -> Self {
        init_logging();

        Self {
            context: Context::new(),
            initial_size: Size::new(800, 600),
            scenes: FastHashMap::default(),
            current_scene: label!("initial"),
        }
    }

    pub fn with_size<S: Into<Size<u32>>>(mut self, size: S) -> Self {
        self.initial_size = size.into();
        self
    }

    pub fn with_initial_scene(mut self, scene: Box<dyn Scene>) -> Self {
        self.scenes.insert(label!("initial"), scene);
        self
    }

    pub fn with_scene(mut self, label: Label, scene: Box<dyn Scene>) -> Self {
        self.scenes.insert(label, scene);
        self
    }

    pub fn add_window(&mut self, label: Label, window: Arc<Window>) {
        let surface = self
            .context
            .render
            .create_surface_for_window(window.clone());

        self.context.windows.insert(label, (window, surface));
    }

    pub fn render(&mut self) {
        for (_, surface_state) in &mut self.context.windows.values_mut() {
            self.context.render.render_to_surface(surface_state);
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::with_user_event().build().expect(":(");

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop
            .run_app(&mut self)
            .expect("Failed to run application");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let attributes = WindowAttributes::default()
            .with_inner_size(PhysicalSize::new(800, 600))
            .with_resizable(false);
        let window = event_loop
            .create_window(attributes.clone())
            .expect("Failed to create window");

        self.add_window(label!("main"), Arc::new(window));

        match self.scenes.get_mut(&self.current_scene) {
            Some(scene) => scene.load(&mut self.context.for_window(&label!("main"))),
            None => {
                fatal!("You must set an initial scene with 'with_initial_scene'!");
                event_loop.exit();
                return;
            }
        };
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some(scene) = self.scenes.get_mut(&self.current_scene) else {
            return;
        };

        let mut context = self.context.for_window(&label!("main"));

        context.time.frame_start();
        context.time.update();

        while let Some(update_start) = context.time.should_tick() {
            scene.fixed_update(&mut context);
            context.time.do_tick(update_start);
        }

        scene.update(&mut context);

        context.input.flush();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                self.render();
            }

            _ => {}
        }
    }
}
