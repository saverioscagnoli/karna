use core::alloc::GlobalAlloc;
use core::alloc::Layout;

use sdl3_sys::SDL_aligned_alloc;
use sdl3_sys::SDL_aligned_free;
use sdl3_sys::SDL_calloc;
use sdl3_sys::SDL_free;
use sdl3_sys::SDL_malloc;
use sdl3_sys::SDL_realloc;

/// SDL_malloc already guarantees
const MAX_NATURAL_ALIGN: usize = 2 * core::mem::size_of::<usize>();

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

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= MAX_NATURAL_ALIGN {
            unsafe { SDL_calloc(1, layout.size()) as *mut u8 }
        } else {
            let ptr = unsafe { self.alloc(layout) };
            if !ptr.is_null() {
                unsafe { core::ptr::write_bytes(ptr, 0, layout.size()) };
            }
            ptr
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.align() <= MAX_NATURAL_ALIGN {
            unsafe { SDL_realloc(ptr as _, new_size) as *mut u8 }
        } else {
            // Over-aligned: must go through alloc + copy + dealloc.
            let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
            let new_ptr = unsafe { self.alloc(new_layout) };
            if !new_ptr.is_null() {
                unsafe {
                    core::ptr::copy_nonoverlapping(ptr, new_ptr, layout.size().min(new_size));
                    self.dealloc(ptr, layout);
                }
            }
            new_ptr
        }
    }
}
