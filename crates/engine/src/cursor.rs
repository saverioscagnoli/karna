use std::ffi::c_void;

use logging::debug;
use logging::error;
use logging::info;
use logging::trace;
use math as m;
use sdl3::SDL_CreateColorCursor;
use sdl3::SDL_CreateSurfaceFrom;
use sdl3::SDL_CreateSystemCursor;
use sdl3::SDL_Cursor;
use sdl3::SDL_DestroyCursor;
use sdl3::SDL_DestroySurface;
use sdl3::SDL_PixelFormat;
use sdl3::SDL_SetCursor;
use sdl3::SDL_SystemCursor;
use utils::FastHashMap;
use utils::Handle;

use crate::Image;
use crate::assets::AssetServer;
use crate::err::sdl_last_error;

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
pub enum Cursor {
    System(SystemCursor),
    Custom(Handle<Image>, m::Vector2<u16>),
}

impl Default for Cursor {
    fn default() -> Self {
        Self::System(SystemCursor::default())
    }
}

#[derive(Default)]
pub struct CursorRegistry {
    pub pending: Option<Cursor>,
    pub active: Option<Cursor>,
    pub cache: FastHashMap<Cursor, *mut SDL_Cursor>,
}

impl CursorRegistry {
    pub fn poll(&mut self, asset_server: &AssetServer) {
        let Some(cursor) = self.pending else {
            return;
        };

        if self.active == Some(cursor) {
            self.pending = None;
            trace!("Requested cursor is already in use: {:?}", cursor);
            return;
        }

        if let Some(&cached) = self.cache.get(&cursor) {
            unsafe {
                if !SDL_SetCursor(cached) {
                    error!("Failed to set cursor {:?}: {}", cursor, sdl_last_error());
                    return;
                }

                trace!("Cache hit: cursor {:?}", cursor);
                self.active = None;
                self.pending = None;
                return;
            }
        }

        let sdl_cursor = match cursor {
            Cursor::System(cursor) => unsafe {
                let cursor = SDL_CreateSystemCursor(cursor.0);

                if cursor.is_null() {
                    error!("Failed to create system cursor: {}", sdl_last_error());
                    return;
                }

                cursor
            },

            Cursor::Custom(image, hotspot) => unsafe {
                if asset_server.is_image_pending(image) {
                    trace!("Cursor image {:?} not ready, retrying next frame.", image);
                    return;
                };

                let hot = hotspot.cast::<i32>();
                let size = asset_server.get_image(image).size.cast::<i32>();
                let rgba = asset_server.get_image_rgba8(image);

                let surface = SDL_CreateSurfaceFrom(
                    size.w(),
                    size.h(),
                    SDL_PixelFormat::SDL_PIXELFORMAT_RGBA32,
                    rgba.as_ptr().cast_mut().cast::<c_void>(),
                    size.w() * 4,
                );

                if surface.is_null() {
                    error!("Failed to create surface for cursor: {}", sdl_last_error());
                    return;
                }

                let cursor = SDL_CreateColorCursor(surface, hot.x, hot.y);
                SDL_DestroySurface(surface);

                if cursor.is_null() {
                    error!("Failed to create custom cursor: {}", sdl_last_error());
                    return;
                }

                cursor
            },
        };

        unsafe {
            if !SDL_SetCursor(sdl_cursor) {
                error!("Failed to set cursor {:?}: {}", cursor, sdl_last_error());
                SDL_DestroyCursor(sdl_cursor);
                return;
            }

            info!("Created new cursor: {:?}", cursor);
        }

        self.active = Some(cursor);
        self.pending = None;
        self.cache.insert(cursor, sdl_cursor);
    }

    pub fn shutdown(self) {
        debug!("Cleaning up remaining cursors.");

        for (kind, ptr) in self.cache {
            unsafe { SDL_DestroyCursor(ptr) };
            debug!("Destroyed cursor {:?}", kind);
        }
    }
}
