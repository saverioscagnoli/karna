use renderer::Renderer;
use std::sync::Arc;
use winit::window::Window;

pub struct Context {
    pub window: Arc<Window>,
    pub render: Renderer,
}

impl Context {
    pub fn new(window: Arc<Window>) -> Self {
        let render = Renderer::_new(window.clone());

        Self { window, render }
    }
}
