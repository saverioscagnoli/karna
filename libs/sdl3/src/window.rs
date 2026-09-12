use core::ffi::CStr;
use core::ptr;

use alloc::ffi::CString;
use sdl3_sys::SDL_DestroyWindow;
use sdl3_sys::SDL_GetWindowID;
use sdl3_sys::SDL_GetWindowSize;
use sdl3_sys::SDL_GetWindowSizeInPixels;
use sdl3_sys::SDL_GetWindowTitle;
use sdl3_sys::SDL_SetWindowSize;
use sdl3_sys::SDL_SetWindowTitle;
use sdl3_sys::SDL_Window;
use sdl3_sys::SDL_WindowID;

use crate::gpu::Device;
use crate::render::Color;

pub type WindowId = SDL_WindowID;

pub struct Window {
    pub(crate) raw: ptr::NonNull<SDL_Window>,
    pub(crate) device: Device,
}

impl Window {
    pub fn id(&self) -> WindowId {
        unsafe { SDL_GetWindowID(self.raw.as_ptr()) }
    }

    pub fn as_ptr(&self) -> *mut SDL_Window {
        self.raw.as_ptr()
    }

    pub fn title(&self) -> &str {
        let ptr = unsafe { SDL_GetWindowTitle(self.raw.as_ptr()) };

        if ptr.is_null() {
            return "Unnamed Window";
        }

        unsafe { CStr::from_ptr(ptr).to_str().unwrap_or("Unnamed Window") }
    }

    pub fn set_title<T>(&self, title: T)
    where
        T: AsRef<str>,
    {
        let c = CString::new(title.as_ref()).unwrap_or_default();
        unsafe { SDL_SetWindowTitle(self.raw.as_ptr(), c.as_ptr()) };
    }

    pub fn size(&self) -> math::Size<u32> {
        let mut size = math::size!(0, 0);
        unsafe { SDL_GetWindowSize(self.raw.as_ptr(), &mut size.width, &mut size.height) };

        size.cast::<u32>()
    }
    pub fn set_size<S>(&self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size = size.into().cast::<i32>();
        unsafe { SDL_SetWindowSize(self.raw.as_ptr(), size.w(), size.h()) };
    }

    pub fn pixel_size(&self) -> math::Size<u32> {
        let mut size = math::size!(0, 0);
        unsafe { SDL_GetWindowSizeInPixels(self.raw.as_ptr(), &mut size.width, &mut size.height) };

        size.cast::<u32>()
    }

    pub fn clear(&self, color: Color) {
        let _ = self.device.clear(&self, color);
    }

    fn destroy(&self) {
        unsafe { SDL_DestroyWindow(self.as_ptr()) }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.device.release_window(&self);
        self.destroy();
    }
}
