use nostd::alloc::boxed::Box;
use sdl3::window::WindowId;

use crate::time::FpsCalculationStrategy;

#[derive(Debug, Clone)]
pub enum WindowEvent {
    SetTitle(Box<str>),
    SetSize(math::Size<u32>),
    SetTargetFPS(u32),
    SetFPSCalculationStrategy(FpsCalculationStrategy),
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    SetTargetTPS(u32),
    Window {
        window: WindowId,
        wevent: WindowEvent,
    },
}
