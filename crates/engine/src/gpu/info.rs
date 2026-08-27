use core::fmt;
use std::ffi::CStr;
use std::ffi::c_char;
use std::ptr;

use sdl3::SDL_GetStringProperty;
use sdl3::SDL_PropertiesID;

pub struct GpuInfo {
    pub name: String,
    pub backend: String,
    pub driver: String,
}

impl fmt::Debug for GpuInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GPU: {} (backend: {}, driver: {})",
            self.name, self.backend, self.driver
        )
    }
}

/// Copies a borrowed C string. SDL owns the memory, so this must not escape
/// as a pointer — copy immediately.
pub unsafe fn owned(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

pub unsafe fn prop(props: SDL_PropertiesID, key: &CStr) -> Option<String> {
    if props == 0 {
        return None;
    }

    unsafe { owned(SDL_GetStringProperty(props, key.as_ptr(), ptr::null())) }
}
