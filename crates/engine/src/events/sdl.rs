//! The engine's own event vocabulary.
//!
//! Nothing here mentions SDL. Translation lives in [`super::poll`], which is
//! the only place allowed to touch `SDL_Event` — so swapping the platform
//! backend (a console port, say) means rewriting that one file, not every
//! caller.
//!
//! Every enum is `#[non_exhaustive]`: SDL defines far more events than a game
//! needs, and the set mapped here will grow. Match with a `_` arm.

use std::path::PathBuf;

use sdl3::SDL_Scancode;

/// Identifies a window without leaking SDL's `SDL_WindowID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WindowId(pub(crate) u32);

impl WindowId {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "window#{}", self.0)
    }
}

/// A physical key position, layout-independent.
///
/// Kept raw here so the events layer does not depend on the input layer's
/// `Key` enum; `Key::from_scancode` is the intended consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Scancode(pub(crate) u32);

impl Scancode {
    pub const fn raw(self) -> SDL_Scancode {
        SDL_Scancode(self.0)
    }
}

/// A virtual key, i.e. what the key means under the current layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keycode(pub(crate) u32);

impl Keycode {
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Modifier keys held when an event fired.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Modifiers(pub(crate) u16);

impl Modifiers {
    // Mirrors SDL's KMOD bits. `poll` asserts these against the headers at
    // compile time, so a change in SDL breaks the build rather than silently
    // reporting the wrong modifier.
    pub(crate) const SHIFT: u16 = 0x0003;
    pub(crate) const CTRL: u16 = 0x00C0;
    pub(crate) const ALT: u16 = 0x0300;
    pub(crate) const GUI: u16 = 0x0C00;
    pub(crate) const CAPS: u16 = 0x2000;
    pub(crate) const NUM: u16 = 0x1000;

    pub const fn shift(self) -> bool {
        self.0 & Self::SHIFT != 0
    }

    pub const fn ctrl(self) -> bool {
        self.0 & Self::CTRL != 0
    }

    pub const fn alt(self) -> bool {
        self.0 & Self::ALT != 0
    }

    /// Command on macOS, Windows key elsewhere.
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
    pub(crate) const fn from_raw(raw: u8) -> Self {
        match raw {
            1 => Self::Left,
            2 => Self::Middle,
            3 => Self::Right,
            4 => Self::X1,
            5 => Self::X2,
            other => Self::Other(other),
        }
    }

    pub(crate) const fn mask(self) -> u8 {
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

/// Top-level event, as produced by [`super::poll`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SDLEvent {
    /// The application was asked to terminate (last window closed, SIGTERM,
    /// platform shutdown).
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

    /// Process lifecycle. Load-bearing on mobile — see [`Lifecycle`].
    Lifecycle(Lifecycle),

    DropFile {
        window: WindowId,
        path: PathBuf,
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
    Moved {
        x: i32,
        y: i32,
    },
    /// Logical size changed. For the backbuffer size use [`Self::PixelSizeChanged`].
    Resized {
        width: u32,
        height: u32,
    },
    /// Size in actual pixels changed — this is what the swapchain cares about,
    /// and on a HiDPI display it differs from `Resized`.
    PixelSizeChanged {
        width: u32,
        height: u32,
    },
    Minimized,
    Maximized,
    Restored,
    MouseEnter,
    MouseLeave,
    FocusGained,
    FocusLost,
    CloseRequested,
    /// The window moved to a display with a different scale factor.
    DisplayScaleChanged,
    EnterFullscreen,
    LeaveFullscreen,
    /// The window is fully hidden by another; a good cue to throttle rendering.
    Occluded,
    /// The usable region changed — mobile notches, rounded corners, gesture bars.
    SafeAreaChanged,
    Destroyed,
}

/// Text as composed by the platform, after layout and IME processing.
///
/// Distinct from [`SDLEvent::Key`]: one keystroke may produce several
/// characters, or none.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TextEvent {
    /// Committed text. Insert at the caret.
    Input { text: String },

    /// In-flight IME composition. Render it underlined at the caret; it is
    /// not part of the document until an `Input` arrives. An empty `text`
    /// means the composition was cancelled — clear the preedit.
    Editing {
        text: String,
        /// Caret position within `text`, in characters. `-1` if unknown.
        cursor: i32,
        /// Selection length within `text`. `-1` if unknown.
        len: i32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub scancode: Scancode,
    pub keycode: Keycode,
    pub modifiers: Modifiers,
    pub pressed: bool,
    /// True when the OS auto-repeat generated this, not a fresh press.
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
        /// 1 for a single click, 2 for a double, and so on.
        clicks: u8,
        x: f32,
        y: f32,
    },
    Wheel {
        /// Already sign-corrected for a "natural scrolling" trackpad.
        x: f32,
        y: f32,
        mouse_x: f32,
        mouse_y: f32,
    },
}

/// Touch input. Coordinates are normalised 0..1 across the touch device, which
/// is *not* the same as window pixels — multiply by the window size.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum TouchEvent {
    Down(Finger),
    Up(Finger),
    Motion(Finger),
    /// The system took over the gesture (a notification, a system swipe).
    /// Treat as an up that should not trigger a tap.
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
    Added {
        id: u32,
    },
    Removed {
        id: u32,
    },
    Button {
        id: u32,
        button: u8,
        pressed: bool,
    },
    Axis {
        id: u32,
        axis: u8,
        /// Normalised to -1..1. Triggers rest at 0 and travel to 1.
        value: f32,
    },
}

/// Process lifecycle transitions.
///
/// On desktop these are rare. On iOS and Android they are mandatory: the OS
/// will terminate an app that keeps rendering after
/// [`Lifecycle::DidEnterBackground`], and both platforms deliver these
/// *synchronously* — SDL calls the event handler from inside the platform
/// callback, so the app must stop drawing before returning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Lifecycle {
    /// Shutdown is imminent and cannot be refused. Save now.
    Terminating,
    /// Free caches or risk being killed.
    LowMemory,
    WillEnterBackground,
    /// Stop rendering and release the GPU surface before returning.
    DidEnterBackground,
    WillEnterForeground,
    DidEnterForeground,
}
