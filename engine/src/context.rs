use crate::{input::Input, time::Time, window::Window};
use renderer::Renderer;
use std::sync::Arc;

pub struct Context {
    pub window: Window,
    pub render: Renderer,
    pub time: Time,
    pub input: Input,
}

impl Context {
    pub fn new(inner: Arc<winit::window::Window>) -> Self {
        let render = Renderer::_new(inner.clone());
        let window = Window::new(inner);
        let time = Time::new();
        let input = Input::new();

        Self {
            window,
            render,
            time,
            input,
        }
    }
}
