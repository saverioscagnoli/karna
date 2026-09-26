use core::ffi::CStr;
use core::mem;
use core::ptr;

use alloc::ffi::CString;
use sdl3_sys::SDL_DestroyWindow;
use sdl3_sys::SDL_DisplayMode;
use sdl3_sys::SDL_GetClosestFullscreenDisplayMode;
use sdl3_sys::SDL_GetDisplayForWindow;
use sdl3_sys::SDL_GetWindowFlags;
use sdl3_sys::SDL_GetWindowID;
use sdl3_sys::SDL_GetWindowOpacity;
use sdl3_sys::SDL_GetWindowSize;
use sdl3_sys::SDL_GetWindowSizeInPixels;
use sdl3_sys::SDL_GetWindowTitle;
use sdl3_sys::SDL_HideWindow;
use sdl3_sys::SDL_MaximizeWindow;
use sdl3_sys::SDL_MinimizeWindow;
use sdl3_sys::SDL_RestoreWindow;
use sdl3_sys::SDL_SetWindowAlwaysOnTop;
use sdl3_sys::SDL_SetWindowBordered;
use sdl3_sys::SDL_SetWindowFocusable;
use sdl3_sys::SDL_SetWindowFullscreen;
use sdl3_sys::SDL_SetWindowFullscreenMode;
use sdl3_sys::SDL_SetWindowKeyboardGrab;
use sdl3_sys::SDL_SetWindowMouseGrab;
use sdl3_sys::SDL_SetWindowOpacity;
use sdl3_sys::SDL_SetWindowRelativeMouseMode;
use sdl3_sys::SDL_SetWindowResizable;
use sdl3_sys::SDL_SetWindowSize;
use sdl3_sys::SDL_SetWindowTitle;
use sdl3_sys::SDL_ShowWindow;
use sdl3_sys::SDL_WINDOW_ALWAYS_ON_TOP;
use sdl3_sys::SDL_WINDOW_BORDERLESS;
use sdl3_sys::SDL_WINDOW_FULLSCREEN;
use sdl3_sys::SDL_WINDOW_HIDDEN;
use sdl3_sys::SDL_WINDOW_HIGH_PIXEL_DENSITY;
use sdl3_sys::SDL_WINDOW_KEYBOARD_GRABBED;
use sdl3_sys::SDL_WINDOW_MAXIMIZED;
use sdl3_sys::SDL_WINDOW_MINIMIZED;
use sdl3_sys::SDL_WINDOW_MOUSE_GRABBED;
use sdl3_sys::SDL_WINDOW_MOUSE_RELATIVE_MODE;
use sdl3_sys::SDL_WINDOW_NOT_FOCUSABLE;
use sdl3_sys::SDL_WINDOW_RESIZABLE;
use sdl3_sys::SDL_WINDOW_TRANSPARENT;
use sdl3_sys::SDL_WINDOW_UTILITY;
use sdl3_sys::SDL_Window;
use sdl3_sys::SDL_WindowFlags;
use sdl3_sys::SDL_WindowID;
use traccia::debug;

use crate::gpu::Device;
use crate::render::Color;

pub type WindowId = SDL_WindowID;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum FullscreenMode {
    #[default]
    Borderless,
    Exclusive {
        width: i32,
        height: i32,
        refresh_rate: f32,
    },
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum WindowState {
    #[default]
    Normal,
    Maximized,
    Minimized,
    Fullscreen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowFlags {
    pub state: WindowState,
    pub hidden: bool,
    pub resizable: bool,
    pub decorated: bool,
    pub always_on_top: bool,
    pub utility: bool,
    pub transparent: bool,
    pub focusable: bool,
    pub high_pixel_density: bool,
    pub grab_mouse: bool,
    pub grab_keyboard: bool,
    pub relative_mouse: bool,
}

impl Default for WindowFlags {
    fn default() -> Self {
        Self {
            state: WindowState::Normal,
            hidden: false,
            resizable: true,
            decorated: true,
            always_on_top: false,
            utility: false,
            transparent: false,
            focusable: true,
            high_pixel_density: true,
            grab_mouse: false,
            grab_keyboard: false,
            relative_mouse: false,
        }
    }
}

impl WindowFlags {
    pub fn to_sdl(&self) -> SDL_WindowFlags {
        let mut flags = SDL_WindowFlags::default();

        match self.state {
            WindowState::Normal => {}
            WindowState::Maximized => flags |= SDL_WINDOW_MAXIMIZED,
            WindowState::Minimized => flags |= SDL_WINDOW_MINIMIZED,
            WindowState::Fullscreen => flags |= SDL_WINDOW_FULLSCREEN,
        }

        if self.hidden {
            flags |= SDL_WINDOW_HIDDEN;
        }

        if self.resizable {
            flags |= SDL_WINDOW_RESIZABLE;
        }

        if !self.decorated {
            flags |= SDL_WINDOW_BORDERLESS;
        }

        if self.always_on_top {
            flags |= SDL_WINDOW_ALWAYS_ON_TOP;
        }

        if self.utility {
            flags |= SDL_WINDOW_UTILITY;
        }

        if self.transparent {
            flags |= SDL_WINDOW_TRANSPARENT;
        }

        if !self.focusable {
            flags |= SDL_WINDOW_NOT_FOCUSABLE;
        }

        if self.high_pixel_density {
            flags |= SDL_WINDOW_HIGH_PIXEL_DENSITY;
        }

        if self.grab_mouse {
            flags |= SDL_WINDOW_MOUSE_GRABBED;
        }

        if self.grab_keyboard {
            flags |= SDL_WINDOW_KEYBOARD_GRABBED;
        }

        if self.relative_mouse {
            flags |= SDL_WINDOW_MOUSE_RELATIVE_MODE;
        }

        flags
    }

    pub fn from_sdl(flags: SDL_WindowFlags) -> Self {
        let has = |flag: SDL_WindowFlags| (flags & flag) == flag;

        let state = if has(SDL_WINDOW_FULLSCREEN) {
            WindowState::Fullscreen
        } else if has(SDL_WINDOW_MINIMIZED) {
            WindowState::Minimized
        } else if has(SDL_WINDOW_MAXIMIZED) {
            WindowState::Maximized
        } else {
            WindowState::Normal
        };

        Self {
            state,
            hidden: has(SDL_WINDOW_HIDDEN),
            resizable: has(SDL_WINDOW_RESIZABLE),
            decorated: !has(SDL_WINDOW_BORDERLESS),
            always_on_top: has(SDL_WINDOW_ALWAYS_ON_TOP),
            utility: has(SDL_WINDOW_UTILITY),
            transparent: has(SDL_WINDOW_TRANSPARENT),
            focusable: !has(SDL_WINDOW_NOT_FOCUSABLE),
            high_pixel_density: has(SDL_WINDOW_HIGH_PIXEL_DENSITY),
            grab_mouse: has(SDL_WINDOW_MOUSE_GRABBED),
            grab_keyboard: has(SDL_WINDOW_KEYBOARD_GRABBED),
            relative_mouse: has(SDL_WINDOW_MOUSE_RELATIVE_MODE),
        }
    }
}

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

    pub fn flags(&self) -> WindowFlags {
        WindowFlags::from_sdl(unsafe { SDL_GetWindowFlags(self.as_ptr()) })
    }

    pub fn title(&self) -> &str {
        let ptr = unsafe { SDL_GetWindowTitle(self.as_ptr()) };

        if ptr.is_null() {
            return "Unnamed Window";
        }

        unsafe { CStr::from_ptr(ptr).to_str().unwrap_or("Unnamed Window") }
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        debug!(
            "Window '{}': setting title to '{}'",
            self.id(),
            title.as_ref()
        );
        let c = CString::new(title.as_ref()).unwrap_or_default();
        unsafe { SDL_SetWindowTitle(self.as_ptr(), c.as_ptr()) };
    }

    pub fn size(&self) -> math::Size<u32> {
        let mut size = math::size!(0, 0);
        unsafe { SDL_GetWindowSize(self.as_ptr(), &mut size.width, &mut size.height) };

        size.cast::<u32>()
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size = size.into().cast::<i32>();
        debug!("Window '{}': setting size to {:?}", self.id(), size);
        unsafe { SDL_SetWindowSize(self.as_ptr(), size.w(), size.h()) };
    }

    pub fn pixel_size(&self) -> math::Size<u32> {
        let mut size = math::size!(0, 0);
        unsafe { SDL_GetWindowSizeInPixels(self.as_ptr(), &mut size.width, &mut size.height) };

        size.cast::<u32>()
    }

    pub fn set_windowed(&mut self) {
        debug!("Window '{}': set fullscreen mode to windowed", self.id());
        unsafe { SDL_SetWindowFullscreen(self.as_ptr(), false) };
    }

    pub fn set_maximized(&mut self) {
        debug!("Window '{}': set to maximized", self.id());
        unsafe { SDL_MaximizeWindow(self.as_ptr()) };
    }

    pub fn set_minimized(&mut self) {
        debug!("Window '{}': set to minimized", self.id());
        unsafe { SDL_MinimizeWindow(self.as_ptr()) };
    }

    pub fn set_fullscreen(&mut self, mode: FullscreenMode) {
        debug!("Window '{}': fullscreen mode set to {:?}", self.id(), mode);

        match mode {
            FullscreenMode::Borderless => unsafe {
                SDL_SetWindowFullscreenMode(self.as_ptr(), ptr::null());
                SDL_SetWindowFullscreen(self.as_ptr(), true);
            },
            FullscreenMode::Exclusive {
                width,
                height,
                refresh_rate,
            } => unsafe {
                let display = SDL_GetDisplayForWindow(self.as_ptr());

                if display == 0 {
                    return;
                }

                let mut closest: SDL_DisplayMode = mem::zeroed();

                if !SDL_GetClosestFullscreenDisplayMode(
                    display,
                    width,
                    height,
                    refresh_rate,
                    true,
                    &mut closest,
                ) {
                    return;
                }

                SDL_SetWindowFullscreenMode(self.as_ptr(), &closest);
                SDL_SetWindowFullscreen(self.as_ptr(), true);
            },
        }
    }

    pub fn restore(&mut self) {
        debug!("Window '{}': restored", self.id());
        unsafe { SDL_RestoreWindow(self.as_ptr()) };
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        debug!("Window '{}': setting hidden to {}", self.id(), hidden);
        if hidden {
            unsafe { SDL_HideWindow(self.as_ptr()) };
        } else {
            unsafe { SDL_ShowWindow(self.as_ptr()) };
        }
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        debug!("Window '{}': setting resizable to {}", self.id(), resizable);
        unsafe { SDL_SetWindowResizable(self.as_ptr(), resizable) };
    }

    pub fn set_decorated(&mut self, decorated: bool) {
        debug!("Window '{}': setting decorated to {}", self.id(), decorated);
        unsafe { SDL_SetWindowBordered(self.as_ptr(), decorated) };
    }

    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        debug!(
            "Window '{}': setting always on top to {}",
            self.id(),
            always_on_top
        );
        unsafe { SDL_SetWindowAlwaysOnTop(self.as_ptr(), always_on_top) };
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        debug!("Window '{}': setting focusable to {}", self.id(), focusable);
        unsafe { SDL_SetWindowFocusable(self.as_ptr(), focusable) };
    }

    pub fn set_mouse_grabbed(&mut self, grabbed: bool) {
        debug!(
            "Window '{}': setting mouse grabbed to {}",
            self.id(),
            grabbed
        );
        unsafe { SDL_SetWindowMouseGrab(self.as_ptr(), grabbed) };
    }

    pub fn set_keyboard_grabbed(&mut self, grabbed: bool) {
        debug!(
            "Window '{}': setting keyboard grabbed to {}",
            self.id(),
            grabbed
        );
        unsafe { SDL_SetWindowKeyboardGrab(self.as_ptr(), grabbed) };
    }

    pub fn set_relative_mouse(&mut self, relative: bool) {
        debug!(
            "Window '{}': setting mouse relative to {}",
            self.id(),
            relative
        );
        unsafe { SDL_SetWindowRelativeMouseMode(self.as_ptr(), relative) };
    }

    pub fn opacity(&self) -> f32 {
        unsafe { SDL_GetWindowOpacity(self.as_ptr()) }
    }

    pub fn set_opacity(&mut self, value: f32) {
        debug!("Window '{}': setting opacity to {}", self.id(), value);
        unsafe { SDL_SetWindowOpacity(self.as_ptr(), value) };
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
