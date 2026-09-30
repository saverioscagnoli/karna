use core::time::Duration;

use nostd::alloc::boxed::Box;
use nostd::collections::Handle;
use sdl3::gpu::PresentMode;
use sdl3::window::FullscreenMode;
use sdl3::window::WindowId;
use sdl3::window::WindowState;

use crate::assets::Audio;
use crate::scene::SceneId;
use crate::time::FpsCalculationStrategy;

#[derive(Debug, Clone)]
pub enum WindowCommand {
    SetTitle(Box<str>),
    SetSize(math::Size<u32>),
    SetOpacity(f32),
    SetPresentMode(PresentMode),
    SetState(WindowState),
    SetFullscreenMode(FullscreenMode),
    SetHidden(bool),
    SetResizable(bool),
    SetDecorated(bool),
    SetAlwaysOnTop(bool),
    SetFocusable(bool),
    SetMouseGrabbed(bool),
    SetKeyboardGrabbed(bool),
    SetRelativeMouse(bool),
    Restore,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeCommand {
    SetTargetFPS(WindowId, u32),
    SetFPSCalculationStrategy(WindowId, FpsCalculationStrategy),
    SetTargetTPS(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum SceneCommand {
    Load(SceneId),
    Activate(SceneId),
    Deactivate(SceneId),
    Unload(SceneId),
}

#[derive(Debug, Clone, Copy)]
pub enum InputCommand {
    Rumble {
        slot: usize,
        low: f32,
        high: f32,
        duration: Duration,
    },
    RumbleTriggers {
        slot: usize,
        left: f32,
        right: f32,
        duration: Duration,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum AudioCommand {
    Play { audio: Handle<Audio> },
}
