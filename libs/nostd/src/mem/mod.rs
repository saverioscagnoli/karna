use core::alloc::GlobalAlloc;
use core::alloc::Layout;

use sdl3_sys::SDL_aligned_alloc;
use sdl3_sys::SDL_aligned_free;
use sdl3_sys::SDL_free;
use sdl3_sys::SDL_malloc;

/// SDL_malloc already guarantees
const MAX_NATURAL_ALIGN: usize = 16;

pub struct SdlAllocator;

unsafe impl GlobalAlloc for SdlAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MAX_NATURAL_ALIGN {
            unsafe { SDL_malloc(layout.size()) as *mut u8 }
        } else {
            unsafe { SDL_aligned_alloc(layout.align(), layout.size()) as *mut u8 }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= MAX_NATURAL_ALIGN {
            unsafe { SDL_free(ptr as _) }
        } else {
            unsafe { SDL_aligned_free(ptr as _) }
        }
    }
}
