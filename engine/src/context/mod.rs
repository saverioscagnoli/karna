mod time;
mod window;

use renderer::Renderer;
use std::sync::Arc;

use crate::context::{time::Time, window::Window};

pub struct Context {
    pub window: Window,
    pub time: Time,
    pub render: Renderer,
}

impl Context {
    pub(crate) fn new(window: Arc<winit::window::Window>) -> Self {
        Self {
            window: Window::from_winit(window.clone()),
            time: Time::new(),
            render: pollster::block_on(Renderer::new(window)),
        }
    }
}
