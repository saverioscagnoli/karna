use alloc::string::String;
use alloc::string::ToString;
use sdl3_sys::SDL_LoadFile;

use core::ops::Deref;
use core::ptr;
use core::slice;

use sdl3_sys::SDL_free;

use crate::path::Path;

pub struct Blob {
    ptr: ptr::NonNull<u8>,
    len: usize,
}

impl Blob {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        self
    }
}

impl Deref for Blob {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl AsRef<[u8]> for Blob {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl Drop for Blob {
    fn drop(&mut self) {
        unsafe { SDL_free(self.ptr.as_ptr().cast()) };
    }
}

unsafe impl Send for Blob {}
unsafe impl Sync for Blob {}

pub fn read(path: impl AsRef<Path>) -> Result<Blob, String> {
    let mut len = 0;

    let ptr = path
        .as_ref()
        .with_c_str(|path| unsafe { SDL_LoadFile(path, &mut len) });

    match ptr::NonNull::new(ptr.cast::<u8>()) {
        Some(ptr) => Ok(Blob { ptr, len }),
        None => Err(sdl3_sys::get_error().to_string()),
    }
}
