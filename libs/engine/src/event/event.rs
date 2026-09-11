use nostd::alloc::boxed::Box;
use sdl3::window::WindowId;

#[derive(Debug, Clone)]
pub enum WindowEvent {
    SetTitle(Box<str>),
    SetTargetFPS(u32),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    SetTargetTPS(u32),
    Window {
        window: WindowId,
        wevent: WindowEvent,
    },
}
