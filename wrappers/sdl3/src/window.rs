use core::ffi::CStr;
use core::mem;
use core::ptr;

use alloc::ffi::CString;
use alloc::format;
use sdl3_sys::SDL_DestroyWindow;
use sdl3_sys::SDL_DisplayMode;
use sdl3_sys::SDL_GPU_SWAPCHAINCOMPOSITION_SDR;
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
use sdl3_sys::SDL_SetGPUSwapchainParameters;
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
use sdl3_sys::SDL_WindowSupportsGPUPresentMode;
use sdl3_sys::get_error;
use traccia::debug;
use traccia::error;

use crate::gpu::Device;
use crate::gpu::PresentMode;
use crate::render::Color;

pub type WindowId = SDL_WindowID;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FullscreenMode {
    #[default]
    Borderless,
    Exclusive {
        width: i32,
        height: i32,
        /// 0.0 uses the desktop refresh rate.
        refresh_rate: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
    pub mouse_grabbed: bool,
    pub keyboard_grabbed: bool,
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
            mouse_grabbed: false,
            keyboard_grabbed: false,
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

        if self.mouse_grabbed {
            flags |= SDL_WINDOW_MOUSE_GRABBED;
        }

        if self.keyboard_grabbed {
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
            mouse_grabbed: has(SDL_WINDOW_MOUSE_GRABBED),
            keyboard_grabbed: has(SDL_WINDOW_KEYBOARD_GRABBED),
            relative_mouse: has(SDL_WINDOW_MOUSE_RELATIVE_MODE),
        }
    }
}

pub struct Window {
    pub(crate) raw: ptr::NonNull<SDL_Window>,
    pub(crate) device: Device,
    pub(crate) present_mode: PresentMode,
}

impl Window {
    fn check(&self, ok: bool, action: &str) -> bool {
        if !ok {
            error!(
                "Window '{}': failed to {}: {}",
                self.id(),
                action,
                get_error()
            );
        }
        ok
    }

    pub fn id(&self) -> WindowId {
        unsafe { SDL_GetWindowID(self.as_ptr()) }
    }

    pub fn as_ptr(&self) -> *mut SDL_Window {
        self.raw.as_ptr()
    }

    pub fn flags(&self) -> WindowFlags {
        WindowFlags::from_sdl(unsafe { SDL_GetWindowFlags(self.as_ptr()) })
    }

    pub fn state(&self) -> WindowState {
        self.flags().state
    }

    pub fn title(&self) -> &str {
        let ptr = unsafe { SDL_GetWindowTitle(self.as_ptr()) };

        if ptr.is_null() {
            return "Unnamed Window";
        }

        unsafe { CStr::from_ptr(ptr) }
            .to_str()
            .unwrap_or("Unnamed Window")
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        let title = title.as_ref();
        debug!("Window '{}': setting title to '{}'", self.id(), title);

        let c = CString::new(title.replace('\0', "")).unwrap_or_default();
        self.check(
            unsafe { SDL_SetWindowTitle(self.as_ptr(), c.as_ptr()) },
            "set title",
        );
    }

    pub fn size(&self) -> math::Size<u32> {
        let mut size = math::size!(0, 0);

        self.check(
            unsafe { SDL_GetWindowSize(self.as_ptr(), &mut size.width, &mut size.height) },
            "get size",
        );

        size.cast::<u32>()
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size = size.into().cast::<i32>();

        debug!("Window '{}': setting size to {:?}", self.id(), size);

        self.check(
            unsafe { SDL_SetWindowSize(self.as_ptr(), size.w(), size.h()) },
            "set size",
        );
    }

    pub fn pixel_size(&self) -> math::Size<u32> {
        let mut size = math::size!(0, 0);

        self.check(
            unsafe { SDL_GetWindowSizeInPixels(self.as_ptr(), &mut size.width, &mut size.height) },
            "get pixel size",
        );

        size.cast::<u32>()
    }

    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    pub fn supports_present_mode(&self, mode: PresentMode) -> bool {
        unsafe {
            SDL_WindowSupportsGPUPresentMode(self.device.as_ptr(), self.as_ptr(), mode.to_sdl())
        }
    }

    pub fn set_present_mode(&mut self, mode: PresentMode) -> bool {
        debug!("Window '{}': setting present mode to {:?}", self.id(), mode);

        if !self.supports_present_mode(mode) {
            error!(
                "Window '{}': present mode {:?} is not supported",
                self.id(),
                mode
            );
            return false;
        }

        let ok = self.check(
            unsafe {
                SDL_SetGPUSwapchainParameters(
                    self.device.as_ptr(),
                    self.as_ptr(),
                    SDL_GPU_SWAPCHAINCOMPOSITION_SDR,
                    mode.to_sdl(),
                )
            },
            "set present mode",
        );

        if ok {
            self.present_mode = mode;
        }

        ok
    }

    pub fn is_hidden(&self) -> bool {
        self.flags().hidden
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        debug!("Window '{}': setting hidden to {}", self.id(), hidden);

        let ok = unsafe {
            if hidden {
                SDL_HideWindow(self.as_ptr())
            } else {
                SDL_ShowWindow(self.as_ptr())
            }
        };

        self.check(ok, if hidden { "hide" } else { "show" });
    }

    pub fn is_maximized(&self) -> bool {
        self.state() == WindowState::Maximized
    }

    pub fn maximize(&mut self) {
        debug!("Window '{}': maximizing", self.id());
        self.check(unsafe { SDL_MaximizeWindow(self.as_ptr()) }, "maximize");
    }

    pub fn is_minimized(&self) -> bool {
        self.state() == WindowState::Minimized
    }

    pub fn minimize(&mut self) {
        debug!("Window '{}': minimizing", self.id());
        self.check(unsafe { SDL_MinimizeWindow(self.as_ptr()) }, "minimize");
    }

    /// Restores a maximized or minimized window to its normal size and position.
    pub fn restore(&mut self) {
        debug!("Window '{}': restoring", self.id());
        self.check(unsafe { SDL_RestoreWindow(self.as_ptr()) }, "restore");
    }

    pub fn is_fullscreen(&self) -> bool {
        self.state() == WindowState::Fullscreen
    }

    pub fn set_fullscreen(&mut self, mode: FullscreenMode) -> bool {
        debug!("Window '{}': entering fullscreen {:?}", self.id(), mode);

        let mut closest: SDL_DisplayMode = unsafe { mem::zeroed() };

        let display_mode: *const SDL_DisplayMode = match mode {
            FullscreenMode::Borderless => ptr::null(),
            FullscreenMode::Exclusive {
                width,
                height,
                refresh_rate,
            } => {
                let display = unsafe { SDL_GetDisplayForWindow(self.as_ptr()) };
                if !self.check(display != 0, "get display") {
                    return false;
                }

                let found = unsafe {
                    SDL_GetClosestFullscreenDisplayMode(
                        display,
                        width,
                        height,
                        refresh_rate,
                        true,
                        &mut closest,
                    )
                };
                let action =
                    format!("find a display mode close to {width}x{height}@{refresh_rate}");
                if !self.check(found, &action) {
                    return false;
                }

                &closest as *const SDL_DisplayMode
            }
        };

        if !self.check(
            unsafe { SDL_SetWindowFullscreenMode(self.as_ptr(), display_mode) },
            "set fullscreen mode",
        ) {
            return false;
        }

        self.check(
            unsafe { SDL_SetWindowFullscreen(self.as_ptr(), true) },
            "enter fullscreen",
        );

        true
    }

    pub fn set_windowed(&mut self) {
        debug!("Window '{}': leaving fullscreen", self.id());
        self.check(
            unsafe { SDL_SetWindowFullscreen(self.as_ptr(), false) },
            "leave fullscreen",
        );
    }

    pub fn is_resizable(&self) -> bool {
        self.flags().resizable
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        debug!("Window '{}': setting resizable to {}", self.id(), resizable);
        self.check(
            unsafe { SDL_SetWindowResizable(self.as_ptr(), resizable) },
            "set resizable",
        );
    }

    pub fn is_decorated(&self) -> bool {
        self.flags().decorated
    }

    pub fn set_decorated(&mut self, decorated: bool) {
        debug!("Window '{}': setting decorated to {}", self.id(), decorated);
        self.check(
            unsafe { SDL_SetWindowBordered(self.as_ptr(), decorated) },
            "set decorated",
        );
    }

    pub fn is_always_on_top(&self) -> bool {
        self.flags().always_on_top
    }

    pub fn set_always_on_top(&mut self, always_on_top: bool) {
        debug!(
            "Window '{}': setting always on top to {}",
            self.id(),
            always_on_top
        );

        self.check(
            unsafe { SDL_SetWindowAlwaysOnTop(self.as_ptr(), always_on_top) },
            "set always on top",
        );
    }

    pub fn is_focusable(&self) -> bool {
        self.flags().focusable
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        debug!("Window '{}': setting focusable to {}", self.id(), focusable);
        self.check(
            unsafe { SDL_SetWindowFocusable(self.as_ptr(), focusable) },
            "set focusable",
        );
    }

    pub fn is_utility(&self) -> bool {
        self.flags().utility
    }

    pub fn is_transparent(&self) -> bool {
        self.flags().transparent
    }

    pub fn is_high_pixel_density(&self) -> bool {
        self.flags().high_pixel_density
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        self.flags().mouse_grabbed
    }

    pub fn set_mouse_grabbed(&mut self, grabbed: bool) {
        debug!(
            "Window '{}': setting mouse grabbed to {}",
            self.id(),
            grabbed
        );

        self.check(
            unsafe { SDL_SetWindowMouseGrab(self.as_ptr(), grabbed) },
            "set mouse grabbed",
        );
    }

    pub fn is_keyboard_grabbed(&self) -> bool {
        self.flags().keyboard_grabbed
    }

    pub fn set_keyboard_grabbed(&mut self, grabbed: bool) {
        debug!(
            "Window '{}': setting keyboard grabbed to {}",
            self.id(),
            grabbed
        );

        self.check(
            unsafe { SDL_SetWindowKeyboardGrab(self.as_ptr(), grabbed) },
            "set keyboard grabbed",
        );
    }

    pub fn is_relative_mouse(&self) -> bool {
        self.flags().relative_mouse
    }

    pub fn set_relative_mouse(&mut self, relative: bool) {
        debug!(
            "Window '{}': setting relative mouse to {}",
            self.id(),
            relative
        );
        self.check(
            unsafe { SDL_SetWindowRelativeMouseMode(self.as_ptr(), relative) },
            "set relative mouse",
        );
    }

    /// Returns 1.0 if the opacity can't be queried.
    pub fn opacity(&self) -> f32 {
        let value = unsafe { SDL_GetWindowOpacity(self.as_ptr()) };
        if value < 0.0 { 1.0 } else { value }
    }

    pub fn set_opacity(&mut self, opacity: f32) {
        let opacity = opacity.clamp(0.0, 1.0);

        debug!("Window '{}': setting opacity to {}", self.id(), opacity);

        self.check(
            unsafe { SDL_SetWindowOpacity(self.as_ptr(), opacity) },
            "set opacity",
        );
    }

    pub fn clear(&self, color: Color) {
        let _ = self.device.clear(self, color);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.device.release_window(self);
        unsafe { SDL_DestroyWindow(self.as_ptr()) };
    }
}
