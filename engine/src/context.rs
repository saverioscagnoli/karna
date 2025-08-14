use winit::window::Window;

pub struct Context {
    pub window: Window,
}

impl Context {
    pub fn new(window: Window) -> Self {
        Self { window }
    }
}
