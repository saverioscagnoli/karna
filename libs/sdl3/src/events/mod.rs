mod poll;

pub use poll::*;

use alloc::string::String;
use sdl3_sys::SDL_Scancode;

use crate::window::WindowId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scancode(pub(crate) u32);

impl Scancode {
    pub const fn raw(self) -> SDL_Scancode {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keycode(pub(crate) u32);

impl Keycode {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(pub(crate) u16);

impl Modifiers {
    pub const SHIFT: u16 = 0x0003;
    pub const CTRL: u16 = 0x00C0;
    pub const ALT: u16 = 0x0300;
    pub const GUI: u16 = 0x0C00;
    pub const CAPS: u16 = 0x2000;
    pub const NUM: u16 = 0x1000;

    pub const fn shift(self) -> bool {
        self.0 & Self::SHIFT != 0
    }

    pub const fn ctrl(self) -> bool {
        self.0 & Self::CTRL != 0
    }

    pub const fn alt(self) -> bool {
        self.0 & Self::ALT != 0
    }

    pub const fn gui(self) -> bool {
        self.0 & Self::GUI != 0
    }

    pub const fn caps_lock(self) -> bool {
        self.0 & Self::CAPS != 0
    }

    pub const fn num_lock(self) -> bool {
        self.0 & Self::NUM != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    X1,
    X2,
    Other(u8),
}

impl MouseButton {
    pub const fn from_raw(raw: u8) -> Self {
        match raw {
            1 => Self::Left,
            2 => Self::Middle,
            3 => Self::Right,
            4 => Self::X1,
            5 => Self::X2,
            other => Self::Other(other),
        }
    }

    pub const fn mask(self) -> u8 {
        1u8 << match self {
            Self::Left => 1,
            Self::Middle => 2,
            Self::Right => 3,
            Self::X1 => 4,
            Self::X2 => 5,
            Self::Other(o) => o,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SdlEvent {
    Quit,
    Window {
        window: WindowId,
        wevent: SDLWindowEvent,
    },
    Key {
        window: WindowId,
        kevent: KeyEvent,
    },
    Mouse {
        window: WindowId,
        mevent: MouseEvent,
    },
    Touch(TouchEvent),
    Gamepad(GamepadEvent),
    Lifecycle(Lifecycle),
    DropFile {
        window: WindowId,
        path: String,
        /// Position within the window, in window coordinates.
        x: f32,
        y: f32,
    },
    DropText {
        window: WindowId,
        text: String,
    },
    Text {
        window: WindowId,
        tevent: TextEvent,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SDLWindowEvent {
    Shown,
    Hidden,
    Exposed,
    Moved { x: i32, y: i32 },
    Resized { width: u32, height: u32 },
    PixelSizeChanged { width: u32, height: u32 },
    Minimized,
    Maximized,
    Restored,
    MouseEnter,
    MouseLeave,
    FocusGained,
    FocusLost,
    CloseRequested,
    DisplayScaleChanged,
    EnterFullscreen,
    LeaveFullscreen,
    Occluded,
    SafeAreaChanged,
    Destroyed,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TextEvent {
    Input { text: String },
    Editing { text: String, cursor: i32, len: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub scancode: Scancode,
    pub keycode: Keycode,
    pub modifiers: Modifiers,
    pub pressed: bool,
    pub repeat: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum MouseEvent {
    Motion {
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    },
    Button {
        button: MouseButton,
        pressed: bool,
        clicks: u8,
        x: f32,
        y: f32,
    },
    Wheel {
        x: f32,
        y: f32,
        mouse_x: f32,
        mouse_y: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum TouchEvent {
    Down(Finger),
    Up(Finger),
    Motion(Finger),
    Canceled(Finger),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Finger {
    pub window: WindowId,
    pub touch_id: u64,
    pub finger_id: u64,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub pressure: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum GamepadEvent {
    Added { id: u32 },
    Removed { id: u32 },
    Button { id: u32, button: u8, pressed: bool },
    Axis { id: u32, axis: u8, value: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Lifecycle {
    Terminating,
    LowMemory,
    WillEnterBackground,
    DidEnterBackground,
    WillEnterForeground,
    DidEnterForeground,
}
