mod input;
mod time;
mod window;

use crate::context::{input::Input, time::Time, window::Window};
use renderer::{Renderer, SharedGPU};
use std::sync::Arc;

pub struct Context {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub render: Renderer,
    pub gpu: Arc<SharedGPU>,
}

impl Context {
    pub(crate) fn new(window: Arc<winit::window::Window>, gpu: Arc<SharedGPU>) -> Self {
        Self {
            window: Window::from_winit(window.clone()),
            time: Time::new(),
            input: Input::new(),
            render: Renderer::new(window, gpu.clone()),
            gpu,
        }
    }
}
