pub mod archive;

use alloc::string::String;
use alloc::string::ToString;
use sdl3_sys::SDL_LoadFile;

use core::ops::Deref;
use core::ptr;
use core::slice;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering;

use sdl3_sys::SDL_free;

use crate::path::Path;

pub use archive::Archive;

/// A file's contents: loaded from disk by SDL, or borrowed from the mounted
/// archive.
pub struct Blob {
    ptr: ptr::NonNull<u8>,
    len: usize,
    /// Allocated by SDL and freed on drop; otherwise `'static` archive data.
    owned: bool,
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
        if self.owned {
            unsafe { SDL_free(self.ptr.as_ptr().cast()) };
        }
    }
}

unsafe impl Send for Blob {}
unsafe impl Sync for Blob {}

static MOUNTED: AtomicPtr<Archive> = AtomicPtr::new(ptr::null_mut());

/// Serve every [`read`] from `archive` instead of the disk, for the rest of
/// the program. Used by bundled games, whose files travel inside the
/// executable.
pub fn mount(archive: &'static Archive) {
    MOUNTED.store(ptr::from_ref(archive).cast_mut(), Ordering::Release);
}

/// The archive set by [`mount`], if any.
pub fn mounted() -> Option<&'static Archive> {
    unsafe { MOUNTED.load(Ordering::Acquire).as_ref() }
}

/// Read a whole file, from the mounted archive if there is one (paths are
/// looked up normalized, see [`archive::normalize`]), otherwise from disk.
pub fn read(path: impl AsRef<Path>) -> Result<Blob, String> {
    let path = path.as_ref();

    if let Some(archive) = mounted() {
        let data = archive
            .get(path.as_str())
            .ok_or_else(|| alloc::format!("{path}: not found in the bundle"))?;

        return Ok(Blob {
            ptr: ptr::NonNull::from(data).cast(),
            len: data.len(),
            owned: false,
        });
    }

    let mut len = 0;

    let ptr = path.with_c_str(|path| unsafe { SDL_LoadFile(path, &mut len) });

    match ptr::NonNull::new(ptr.cast::<u8>()) {
        Some(ptr) => Ok(Blob {
            ptr,
            len,
            owned: true,
        }),
        None => Err(sdl3_sys::get_error().to_string()),
    }
}
