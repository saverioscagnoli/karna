mod context;
mod handle;
mod pacer;
mod state;
mod time;

use std::ffi::CStr;
use std::ffi::CString;

use logging::debug;
use logging::error;
use logging::fatal;
use math as m;
use sdl3::SDL_CreateWindow;
use sdl3::SDL_DestroyWindow;
use sdl3::SDL_GetWindowFlags;
use sdl3::SDL_GetWindowID;
use sdl3::SDL_GetWindowSize;
use sdl3::SDL_GetWindowTitle;
use sdl3::SDL_SetWindowResizable;
use sdl3::SDL_SetWindowSize;
use sdl3::SDL_SetWindowTitle;
use sdl3::SDL_WINDOW_RESIZABLE;
use sdl3::SDL_Window;

use crate::err::sdl_last_error;
use crate::events::WindowId;

pub use context::DrawContext;
pub use context::LoadContext;
pub use context::UpdateContext;
pub use context::UserContext;
pub use handle::WindowHandle;
pub use pacer::FpsCalcStrategy;
pub use pacer::FramePacer;
pub use pacer::PaceMode;
pub use state::SceneSlot;
pub use state::UpdatePhase;
pub use state::WindowState;
pub use time::Time;

pub struct WindowEntry {
    pub window: Window,
    pub state: WindowState,
}

pub struct Window {
    raw: *mut SDL_Window,
}

impl Window {
    pub fn new(title: &str, size: m::Size<u32>, resizable: bool) -> Self {
        let title = CString::new(title).expect("Title already contains null-terminator");
        let size = size.cast::<i32>();
        let window = unsafe {
            SDL_CreateWindow(
                title.as_ptr(),
                size.w(),
                size.h(),
                if resizable { SDL_WINDOW_RESIZABLE } else { 0 },
            )
        };

        if window.is_null() {
            fatal!("Failed to create window: {}", sdl_last_error());
        }

        Self { raw: window }
    }

    pub(crate) fn raw(&self) -> *mut SDL_Window {
        self.raw
    }

    pub fn id(&self) -> WindowId {
        WindowId(unsafe { SDL_GetWindowID(self.raw) })
    }

    pub fn title(&self) -> &str {
        unsafe {
            let ptr = SDL_GetWindowTitle(self.raw);

            if ptr.is_null() {
                return "Unnamed Window";
            }

            CStr::from_ptr(ptr).to_str().unwrap_or("Unnamed Window")
        }
    }

    pub fn set_title<T>(&self, title: T)
    where
        T: AsRef<str>,
    {
        let title = CString::new(title.as_ref()).expect("Title contains null-terminator");

        if !unsafe { SDL_SetWindowTitle(self.raw, title.as_ptr()) } {
            error!("Failed to set window title: {}", sdl_last_error());
        }
    }

    pub fn size(&self) -> m::Size<u32> {
        let mut w = 0;
        let mut h = 0;

        unsafe { SDL_GetWindowSize(self.raw, &mut w, &mut h) };

        m::Size::new(w, h).cast::<u32>()
    }

    pub fn set_size<S>(&self, size: S)
    where
        S: Into<m::Size<u32>>,
    {
        let size: m::Size<u32> = size.into();
        let size = size.cast::<i32>();

        if !unsafe { SDL_SetWindowSize(self.raw, size.w(), size.h()) } {
            error!("Failed to set window size: {}", sdl_last_error());
        }
    }

    pub fn is_resizable(&self) -> bool {
        unsafe { SDL_GetWindowFlags(self.raw) & SDL_WINDOW_RESIZABLE != 0 }
    }

    pub fn set_resizable(&self, value: bool) {
        if !unsafe { SDL_SetWindowResizable(self.raw, value) } {
            error!("Failed to set window resizable flag: {}", sdl_last_error());
        }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        debug!("Dropping '{}' (\"{}\")", self.id(), self.title());

        unsafe {
            SDL_DestroyWindow(self.raw);
        }
    }
}
