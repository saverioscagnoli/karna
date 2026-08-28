use math as m;
use sdl3::SDL_SystemCursor;
use utils::Handle;

use crate::Image;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemCursor(pub(crate) SDL_SystemCursor);

impl Default for SystemCursor {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl SystemCursor {
    pub const COUNT: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_COUNT);
    pub const CROSSHAIR: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_CROSSHAIR);
    pub const DEFAULT: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_DEFAULT);
    pub const EW_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_EW_RESIZE);
    pub const E_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_E_RESIZE);
    pub const MOVE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_MOVE);
    pub const NESW_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_NESW_RESIZE);
    pub const NE_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_NE_RESIZE);
    pub const NOT_ALLOWED: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_NOT_ALLOWED);
    pub const NS_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_NS_RESIZE);
    pub const NWSE_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_NWSE_RESIZE);
    pub const NW_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_NW_RESIZE);
    pub const N_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_N_RESIZE);
    pub const POINTER: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_POINTER);
    pub const PROGRESS: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_PROGRESS);
    pub const SE_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_SE_RESIZE);
    pub const SW_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_SW_RESIZE);
    pub const S_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_S_RESIZE);
    pub const TEXT: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_TEXT);
    pub const WAIT: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_WAIT);
    pub const W_RESIZE: Self = Self(SDL_SystemCursor::SDL_SYSTEM_CURSOR_W_RESIZE);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorKind {
    System(SystemCursor),
    Custom(Handle<Image>, m::Vector2<u16>),
}

impl Default for CursorKind {
    fn default() -> Self {
        Self::System(SystemCursor::default())
    }
}
