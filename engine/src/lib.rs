mod context;

use crate::context::Context;
use math::Size;
use renderer::Color;
use std::sync::Arc;
use traccia::{Color as TColor, Colorize, LogLevel, Style, info};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

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
}

impl ApplicationHandler<Context> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size: PhysicalSize<u32> = self.window_size.into();
        let attributes = Window::default_attributes().with_inner_size(size);
        let window = event_loop
            .create_window(attributes)
            .expect("Failed to create window");

        let window = Arc::new(window);

        let context = Context::new(window);
        let info = context.render.info();

        info!("backend: {}", info.backend);
        info!("device type: {:?}", info.device_type);
        info!("driver: {}", info.driver_info);
        info!("card: {}", info.name);

        context.window.request_redraw();
        self.context = Some(context);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: Context) {
        self.context = Some(event);
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

            WindowEvent::RedrawRequested => {
                context.render._clear();
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                context.render.set_clear_color(Color::Black);
                context.render.set_draw_color(Color::Magenta);
                context
                    .render
                    .fill_triangle([100.0, 300.0], [150.0, 200.0], [200.0, 300.0]);

                context.render.set_draw_color(Color::Cyan);
                context.render.fill_rect([10, 10], (50, 50));

                context.window.request_redraw();
                context.render.render();

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
            }
            _ => (),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            context: None,
            window_size: Size::new(800, 600),
        }
    }

    pub fn with_size<S: Into<Size<u32>>>(mut self, size: S) -> Self {
        self.window_size = size.into();
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

        event_loop
            .run_app(&mut self)
            .expect("Failed to run application");
        Ok(())
    }
}
