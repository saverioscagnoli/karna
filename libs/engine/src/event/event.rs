use nostd::alloc::boxed::Box;
use sdl3::window::FullscreenMode;
use sdl3::window::WindowId;
use sdl3::window::WindowState;

use crate::time::FpsCalculationStrategy;

#[derive(Debug, Clone)]
pub enum WindowEvent {
    SetTitle(Box<str>),
    SetSize(math::Size<u32>),
    SetTargetFPS(u32),
    SetFPSCalculationStrategy(FpsCalculationStrategy),
    SetResizable(bool),
    SetDecorated(bool),
    SetAlwaysOnTop(bool),
    SetOpacity(f32),
    SetFocusable(bool),
    SetMouseGrabbed(bool),
    SetKeyboardGrabbed(bool),
    SetWindowState(WindowState),
    SetFullscreen(FullscreenMode),
    SetHidden(bool),
    SetRelativeMouse(bool),
    Restore,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    SetTargetTPS(u32),
    Window {
        window: WindowId,
        wevent: WindowEvent,
    },
}
