use std::ffi::CStr;
use std::ffi::CString;
use std::rc::Rc;

use logging::debug;
use logging::error;
use logging::fatal;
use sdl3::SDL_ClaimWindowForGPUDevice;
use sdl3::SDL_CreateWindow;
use sdl3::SDL_DestroyWindow;
use sdl3::SDL_GetWindowID;
use sdl3::SDL_GetWindowSize;
use sdl3::SDL_GetWindowTitle;
use sdl3::SDL_ReleaseWindowFromGPUDevice;
use sdl3::SDL_SetWindowResizable;
use sdl3::SDL_SetWindowSize;
use sdl3::SDL_SetWindowTitle;
use sdl3::SDL_WINDOW_BORDERLESS;
use sdl3::SDL_WINDOW_RESIZABLE;
use sdl3::SDL_Window;

use crate::err::SDL_LastError;
use crate::gpu::Gpu;
use crate::window::WindowId;

pub struct PlatformWindow {
    raw: *mut SDL_Window,
    gpu: Rc<Gpu>,
}

impl PlatformWindow {
    pub fn new(title: &str, size: math::Size<u32>, gpu: Rc<Gpu>) -> Self {
        let title = CString::new(title).expect("Title already null-terminated");
        let window = unsafe {
            SDL_CreateWindow(
                title.as_ptr(),
                size.width as i32,
                size.height as i32,
                SDL_WINDOW_RESIZABLE,
            )
        };

        if window.is_null() {
            fatal!("{}", SDL_LastError());
        }

        unsafe {
            let success = SDL_ClaimWindowForGPUDevice(gpu.raw(), window);

            if !success {
                fatal!("Failed to claim window for gpu device: {}", SDL_LastError());
            }
        }

        Self { raw: window, gpu }
    }

    pub(crate) fn raw(&self) -> *mut SDL_Window {
        self.raw
    }

    pub fn id(&self) -> WindowId {
        unsafe { SDL_GetWindowID(self.raw) }
    }

    pub fn title(&self) -> &str {
        unsafe {
            let ptr = SDL_GetWindowTitle(self.raw);

            if ptr.is_null() {
                return "unnamed window";
            }

            CStr::from_ptr(ptr).to_str().unwrap_or("unnamed window")
        }
    }

    pub fn set_title<T>(&self, title: T)
    where
        T: AsRef<str>,
    {
        let title = CString::new(title.as_ref()).expect("Title already contains null terminator");
        unsafe {
            let success = SDL_SetWindowTitle(self.raw, title.as_ptr());

            if !success {
                error!("Failed to set window title: {}", SDL_LastError());
            }
        }
    }

    pub fn size(&self) -> math::Size<u32> {
        let mut w = 0;
        let mut h = 0;

        unsafe { SDL_GetWindowSize(self.raw, &mut w, &mut h) };

        math::Size::new(w as u32, h as u32)
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size: math::Size<u32> = size.into();

        unsafe {
            let success = SDL_SetWindowSize(self.raw, size.width as i32, size.height as i32);

            if !success {
                error!("Failed to set window size: {}", SDL_LastError());
            }
        }
    }

    pub fn set_resizable(&self, v: bool) {
        unsafe {
            let success = SDL_SetWindowResizable(self.raw, v);

            if !success {
                error!("Failed to set window resizable prop: {}", SDL_LastError());
            }
        }
    }
}

impl Drop for PlatformWindow {
    fn drop(&mut self) {
        unsafe {
            SDL_ReleaseWindowFromGPUDevice(self.gpu.raw(), self.raw);
            SDL_DestroyWindow(self.raw);
        }
        debug!("Dropping window.");
    }
}
