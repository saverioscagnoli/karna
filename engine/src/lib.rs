use std::sync::Arc;

use renderer::Renderer;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Debug, Default)]
pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let size = PhysicalSize::new(1280, 720);
        let attributes = Window::default_attributes()
            .with_inner_size(size)
            .with_resizable(false);
        let window = event_loop.create_window(attributes).expect(":(");

        window.request_redraw();
        let window = Arc::new(window);

        self.window = Some(window.clone());
        self.renderer = Some(pollster::block_on(Renderer::new(window)).unwrap());
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(renderer) = &mut self.renderer else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::RedrawRequested => {
                renderer.fill_rect([10.0, 10.0].into(), [50.0, 50.0].into());
                renderer.end_frame();
            }

            _ => {}
        }
    }
}

impl App {
    pub fn new() -> Self {
        App {
            window: None,
            renderer: None,
        }
    }

    pub fn run(mut self) {
        let event_loop = EventLoop::with_user_event().build().expect(":(");

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(&mut self).expect(":(");
    }
}
