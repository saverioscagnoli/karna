use core::ffi::CStr;
use core::ptr;

use alloc::ffi::CString;
use sdl3_sys::SDL_DestroyWindow;
use sdl3_sys::SDL_GetWindowFlags;
use sdl3_sys::SDL_GetWindowID;
use sdl3_sys::SDL_GetWindowOpacity;
use sdl3_sys::SDL_GetWindowSize;
use sdl3_sys::SDL_GetWindowSizeInPixels;
use sdl3_sys::SDL_GetWindowTitle;
use sdl3_sys::SDL_SetWindowAlwaysOnTop;
use sdl3_sys::SDL_SetWindowBordered;
use sdl3_sys::SDL_SetWindowFocusable;
use sdl3_sys::SDL_SetWindowKeyboardGrab;
use sdl3_sys::SDL_SetWindowMouseGrab;
use sdl3_sys::SDL_SetWindowOpacity;
use sdl3_sys::SDL_SetWindowResizable;
use sdl3_sys::SDL_SetWindowSize;
use sdl3_sys::SDL_SetWindowTitle;
use sdl3_sys::SDL_WINDOW_ALWAYS_ON_TOP;
use sdl3_sys::SDL_WINDOW_BORDERLESS;
use sdl3_sys::SDL_WINDOW_FULLSCREEN;
use sdl3_sys::SDL_WINDOW_HIGH_PIXEL_DENSITY;
use sdl3_sys::SDL_WINDOW_KEYBOARD_GRABBED;
use sdl3_sys::SDL_WINDOW_MOUSE_GRABBED;
use sdl3_sys::SDL_WINDOW_NOT_FOCUSABLE;
use sdl3_sys::SDL_WINDOW_RESIZABLE;
use sdl3_sys::SDL_WINDOW_TRANSPARENT;
use sdl3_sys::SDL_Window;
use sdl3_sys::SDL_WindowFlags;
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

    pub fn flags(&self) -> SDL_WindowFlags {
        unsafe { SDL_GetWindowFlags(self.as_ptr()) }
    }

    pub fn has_flag(&self, flag: SDL_WindowFlags) -> bool {
        self.flags() & flag == flag
    }

    pub fn title(&self) -> &str {
        let ptr = unsafe { SDL_GetWindowTitle(self.as_ptr()) };

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
        unsafe { SDL_SetWindowTitle(self.as_ptr(), c.as_ptr()) };
    }

    pub fn size(&self) -> math::Size<u32> {
        let mut size = math::size!(0, 0);
        unsafe { SDL_GetWindowSize(self.as_ptr(), &mut size.width, &mut size.height) };

        size.cast::<u32>()
    }
    pub fn set_size<S>(&self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size = size.into().cast::<i32>();
        unsafe { SDL_SetWindowSize(self.as_ptr(), size.w(), size.h()) };
    }

    pub fn pixel_size(&self) -> math::Size<u32> {
        let mut size = math::size!(0, 0);
        unsafe { SDL_GetWindowSizeInPixels(self.as_ptr(), &mut size.width, &mut size.height) };

        size.cast::<u32>()
    }

    pub fn is_resizable(&self) -> bool {
        self.has_flag(SDL_WINDOW_RESIZABLE)
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        unsafe { SDL_SetWindowResizable(self.as_ptr(), resizable) };
    }

    pub fn is_decorated(&self) -> bool {
        !self.has_flag(SDL_WINDOW_BORDERLESS) && !self.has_flag(SDL_WINDOW_FULLSCREEN)
    }

    pub fn set_decorated(&mut self, decorated: bool) {
        unsafe { SDL_SetWindowBordered(self.as_ptr(), decorated) };
    }

    pub fn is_always_on_top(&self) -> bool {
        self.has_flag(SDL_WINDOW_ALWAYS_ON_TOP)
    }

    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        unsafe { SDL_SetWindowAlwaysOnTop(self.as_ptr(), always_on_top) };
    }

    pub fn is_transparent(&self) -> bool {
        self.has_flag(SDL_WINDOW_TRANSPARENT)
    }

    pub fn opacity(&self) -> f32 {
        unsafe { SDL_GetWindowOpacity(self.as_ptr()) }
    }

    pub fn set_opacity(&mut self, value: f32) {
        unsafe { SDL_SetWindowOpacity(self.as_ptr(), value) };
    }

    pub fn is_focusable(&self) -> bool {
        !self.has_flag(SDL_WINDOW_NOT_FOCUSABLE)
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        unsafe { SDL_SetWindowFocusable(self.as_ptr(), focusable) };
    }

    pub fn is_high_pixel_density(&self) -> bool {
        self.has_flag(SDL_WINDOW_HIGH_PIXEL_DENSITY)
    }

    pub fn mouse_grabbed(&self) -> bool {
        self.has_flag(SDL_WINDOW_MOUSE_GRABBED)
    }

    pub fn set_mouse_grabbed(&mut self, grabbed: bool) {
        unsafe { SDL_SetWindowMouseGrab(self.as_ptr(), grabbed) };
    }

    pub fn keyboard_grabbed(&self) -> bool {
        self.has_flag(SDL_WINDOW_KEYBOARD_GRABBED)
    }

    pub fn set_keyboard_grabbed(&mut self, grabbed: bool) {
        unsafe { SDL_SetWindowKeyboardGrab(self.as_ptr(), grabbed) };
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
