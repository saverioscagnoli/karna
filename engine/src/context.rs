use renderer::Renderer;
use std::sync::Arc;
use winit::window::Window;

use crate::{input::Input, time::Time};

pub struct Context {
    pub window: Arc<Window>,
    pub render: Renderer,
    pub time: Time,
    pub input: Input,
}

impl Context {
    pub fn new(window: Arc<Window>) -> Self {
        let render = Renderer::_new(window.clone());
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
