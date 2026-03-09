use std::sync::atomic::{AtomicU32, AtomicUsize, Ordering};

use macros::{Get, With};
use math::Size;
use winit::window::WindowAttributes;

use crate::App;

static ID_AUTOINC: AtomicUsize = AtomicUsize::new(1);

fn window_id() -> usize {
    ID_AUTOINC.fetch_add(1, Ordering::Relaxed)
}

#[derive(Get, With)]
pub struct WindowBuilder {
    #[get(copied)]
    id: usize,

    #[with(into)]
    size: Size<u32>,

    #[with(into)]
    title: String,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            id: window_id(),
            size: Size::new(800, 600),
            title: String::from("My Window"),
        }
    }
}

impl WindowBuilder {
    pub fn new() -> Self {
        WindowBuilder::default()
    }

    pub(crate) fn attributes(self) -> WindowAttributes {
        WindowAttributes::default()
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.size.width,
                self.size.height,
            ))
            .with_title(self.title)
    }
}

pub struct AppBuilder {
    windows: Vec<WindowBuilder>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
        }
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_window(mut self, builder: WindowBuilder) -> Self {
        self.windows.push(builder);
        self
    }

    pub fn build(self) -> App {
        let mut app = App::new();

        for b in self.windows {
            app.queue_window(b);
        }

        app
    }
}
