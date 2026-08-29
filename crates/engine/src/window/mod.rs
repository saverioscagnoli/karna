mod context;
mod handle;
mod pacer;
mod state;
mod text;
mod time;

use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;

use logging::debug;
use logging::error;
use logging::fatal;
use math as m;
use sdl3::SDL_ClaimWindowForGPUDevice;
use sdl3::SDL_CreateWindow;
use sdl3::SDL_DestroyWindow;
use sdl3::SDL_GetWindowFlags;
use sdl3::SDL_GetWindowID;
use sdl3::SDL_GetWindowSize;
use sdl3::SDL_GetWindowTitle;
use sdl3::SDL_Rect;
use sdl3::SDL_ReleaseWindowFromGPUDevice;
use sdl3::SDL_SetTextInputArea;
use sdl3::SDL_SetWindowResizable;
use sdl3::SDL_SetWindowSize;
use sdl3::SDL_SetWindowTitle;
use sdl3::SDL_StartTextInput;
use sdl3::SDL_StopTextInput;
use sdl3::SDL_WINDOW_RESIZABLE;
use sdl3::SDL_Window;

use crate::err::sdl_last_error;
use crate::events::WindowId;
use crate::gpu::Device;

pub use context::DrawContext;
pub use context::ForContextMut;
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
pub use text::TextHandle;
pub use time::Time;

pub struct WindowEntry {
    pub window: Window,
    pub state: WindowState,
}

pub struct Window {
    raw: *mut SDL_Window,
    gpu: Device,
}

impl Window {
    pub fn new(title: &str, size: m::Size<u32>, resizable: bool, gpu: Device) -> Self {
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

        let this = Self { raw: window, gpu };

        unsafe {
            let success = SDL_ClaimWindowForGPUDevice(this.gpu.raw(), window);

            if !success {
                fatal!(
                    "Failed to claim window for gpu device: {}",
                    sdl_last_error()
                );
            }

            debug!("GPU device claimed {:?} (\"{}\")", this.id(), this.title());
        }

        this
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

    pub fn start_text_input(&self) {
        if !unsafe { SDL_StartTextInput(self.raw) } {
            error!("Failed to start text input: {}", sdl_last_error());
        }
    }

    pub fn stop_text_input(&self) {
        if !unsafe { SDL_StopTextInput(self.raw) } {
            error!("Failed to stop text input: {}", sdl_last_error());
        }
    }

    pub fn set_text_input_area(&self, origin: m::Vector2<i32>, size: m::Size<u32>, cursor: i32) {
        let size = size.cast::<i32>();
        let rect = SDL_Rect {
            x: origin.x,
            y: origin.y,
            w: size.w(),
            h: size.h(),
        };

        if !unsafe { SDL_SetTextInputArea(self.raw, &rect, cursor) } {
            error!("Failed to set text input area: {}", sdl_last_error());
        }
    }

    pub fn clear_text_input_area(&self) {
        if !unsafe { SDL_SetTextInputArea(self.raw, ptr::null(), 0) } {
            error!("Failed to clear text input area: {}", sdl_last_error());
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
            SDL_ReleaseWindowFromGPUDevice(self.gpu.raw(), self.raw);
            SDL_DestroyWindow(self.raw);
        }
    }
}
