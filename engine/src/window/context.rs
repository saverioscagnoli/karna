use sdl3::video::Window;

use crate::window::WindowHandle;

pub struct WindowContext {
    pub window: WindowHandle,
}

pub struct ContextRef<'a> {
    pub window: &'a mut Window,
}
