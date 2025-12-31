use std::{
    alloc::{GlobalAlloc, System},
    sync::atomic::{AtomicUsize, Ordering},
};

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

pub struct TrackingAllocator;

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let ret = unsafe { System.alloc(layout) };

        if !ret.is_null() {
            let current = CURRENT.fetch_add(layout.size(), Ordering::Relaxed);

            PEAK.fetch_max(current, Ordering::Relaxed);
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }

        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe {
            System.dealloc(ptr, layout);
        }

        if !ptr.is_null() {
            ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
            DEALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
            CURRENT.fetch_sub(layout.size(), Ordering::Relaxed);
        }
    }
}

#[inline]
pub fn allocated_bytes() -> usize {
    ALLOCATED.load(Ordering::Relaxed)
}

#[inline]
pub fn deallocated_bytes() -> usize {
    DEALLOCATED.load(Ordering::Relaxed)
}

#[inline]
pub fn current_bytes() -> usize {
    CURRENT.load(Ordering::Relaxed)
}

#[inline]
pub fn peak_bytes() -> usize {
    PEAK.load(Ordering::Relaxed)
}
