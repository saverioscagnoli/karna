use core::ffi::c_void;
use core::ptr;
use core::ptr::NonNull;
use core::slice;

use sdl3_image_sys::IMG_Load_IO;
use sdl3_sys::SDL_ConvertSurface;
use sdl3_sys::SDL_DestroySurface;
use sdl3_sys::SDL_IOFromConstMem;

use sdl3_sys::SDL_PixelFormat;
use sdl3_sys::SDL_Surface;
use sdl3_sys::SdlError;
use sdl3_sys::get_error;

#[cfg(target_endian = "little")]
const PIXELFORMAT_RGBA32: SDL_PixelFormat = sdl3_sys::SDL_PIXELFORMAT_ABGR8888;

#[cfg(target_endian = "big")]
const PIXELFORMAT_RGBA32: SDL_PixelFormat = sdl3_sys::SDL_PIXELFORMAT_RGBA8888;

pub struct DecodedImage {
    surface: ptr::NonNull<SDL_Surface>,
}

impl DecodedImage {
    fn into_rgba32(self) -> Result<Self, SdlError> {
        if self.surf().format == PIXELFORMAT_RGBA32 {
            return Ok(self);
        }
        let converted = unsafe { SDL_ConvertSurface(self.surface.as_ptr(), PIXELFORMAT_RGBA32) };
        // `self` drops here, destroying the original surface.
        Self::from_raw(converted)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SdlError> {
        if bytes.is_empty() {
            return Err(get_error());
        }

        let io = unsafe { SDL_IOFromConstMem(bytes.as_ptr() as *const c_void, bytes.len()) };
        if io.is_null() {
            return Err(get_error());
        }

        let raw = unsafe { IMG_Load_IO(io, true) };
        let decoded = Self::from_raw(raw)?;

        decoded.into_rgba32()
    }

    fn from_raw(ptr: *mut SDL_Surface) -> Result<Self, SdlError> {
        NonNull::new(ptr)
            .map(|surface| Self { surface })
            .ok_or_else(get_error)
    }

    pub fn raw(&self) -> *mut SDL_Surface {
        self.surface.as_ptr()
    }

    #[inline]
    fn surf(&self) -> &SDL_Surface {
        unsafe { self.surface.as_ref() }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.surf().w as u32
    }

    #[inline]
    pub fn heigth(&self) -> u32 {
        self.surf().h as u32
    }

    #[inline]
    pub fn pixels(&self) -> &[u8] {
        let s = self.surf();
        debug_assert_eq!(s.pitch as usize, s.w as usize * 4);

        unsafe { slice::from_raw_parts(s.pixels as *const u8, (s.w * s.h * 4) as usize) }
    }
}

unsafe impl Send for DecodedImage {}

impl Drop for DecodedImage {
    fn drop(&mut self) {
        unsafe { SDL_DestroySurface(self.surface.as_ptr()) }
    }
}
