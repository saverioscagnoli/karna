use std::collections::HashMap;

use winit::window::WindowId;

use crate::builder::AppBuilder;
use crate::window::WindowHandle;

mod builder;
mod context;
mod scene;
mod window;

pub struct App {
    windows: HashMap<WindowId, WindowHandle>,
}

impl App {
    pub(crate) fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }

    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }
}
