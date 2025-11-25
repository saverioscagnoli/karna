use std::sync::Arc;

use renderer::Renderer;

use crate::{input::Input, time::Time};

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
