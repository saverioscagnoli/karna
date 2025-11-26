use crate::{input::Input, time::Time};
use renderer::Renderer;
use std::sync::Arc;

pub struct Context {
    pub input: Input,
    pub time: Time,
    pub render: Renderer,
}

impl Context {
    pub(crate) fn new(window: Arc<winit::window::Window>) -> Self {
        Self {
            input: Input::new(),
            time: Time::new(),
            render: pollster::block_on(Renderer::new(window)).unwrap(),
        }
    }
}
