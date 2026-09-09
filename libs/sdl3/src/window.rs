use core::ptr;

use alloc::ffi::CString;
use alloc::string::String;
use sdl3_sys::SDL_CreateWindow;
use sdl3_sys::SDL_Window;
use sdl3_sys::SdlError;

pub struct Window(ptr::NonNull<SDL_Window>);

pub fn create_window<T, S>(title: T, size: S) -> Result<Window, SdlError>
where
    T: Into<String>,
    S: Into<math::Size<u32>>,
{
    let title =
        CString::new(title.into()).map_err(|_| SdlError::new("String already has nullterm"))?;

    let size = size.into().cast::<i32>();

    unsafe { SDL_CreateWindow(title, w, h, flags) }
}

impl Window {}
