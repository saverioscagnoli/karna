use alloc::string::String;
use alloc::string::ToString;
use sdl3_sys::SDL_LoadFile;

use core::ffi;
use core::ops::Deref;
use core::ptr;
use core::slice;

use sdl3_sys::SDL_free;

pub struct CPath {
    buf: [u8; 512],
    len: usize,
}

impl CPath {
    pub fn new(s: &str) -> Result<Self, String> {
        let b = s.as_bytes();

        if b.len() >= 512 {
            return Err(String::from("Path too long"));
        }

        if b.contains(&0) {
            return Err(String::from("Interior null"));
        }

        let mut buf = [0u8; 512];
        buf[..b.len()].copy_from_slice(b);
        Ok(Self { buf, len: b.len() })
    }

    pub fn as_ptr(&self) -> *const ffi::c_char {
        self.buf.as_ptr().cast()
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

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

pub fn read(path: &str) -> Result<Blob, String> {
    let path = CPath::new(path)?;
    let mut len = 0;

    let ptr = unsafe { SDL_LoadFile(path.as_ptr(), &mut len) };

    match ptr::NonNull::new(ptr.cast::<u8>()) {
        Some(ptr) => Ok(Blob { ptr, len }),
        None => Err(sdl3_sys::get_error().to_string()),
    }
}
