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
    pub fn new(inner: Arc<winit::window::Window>) -> Result<Self, Box<dyn std::error::Error>> {
        let render = pollster::block_on(Renderer::_new(inner.clone()))?;
        let window = Window::new(inner);
        let time = Time::new();
        let input = Input::new();

        Ok(Self {
            window,
            render,
            time,
            input,
        })
    }
}
