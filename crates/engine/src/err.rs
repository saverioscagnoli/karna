use std::ffi::CStr;

use sdl3::SDL_GetError;

pub fn sdl_last_error() -> String {
    unsafe {
        let ptr = SDL_GetError();
        if ptr.is_null() {
            "Unknown SDL error".to_owned()
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }
}
