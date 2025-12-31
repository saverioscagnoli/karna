use macros::Get;

use crate::allocator::{allocated_bytes, current_bytes, deallocated_bytes, peak_bytes};

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
#[derive(Get)]
pub struct MemoryStats {
    #[get(copied)]
    pub(crate) allocated: usize,

    #[get(copied)]
    pub(crate) deallocated: usize,

    #[get(copied)]
    pub(crate) current: usize,

    #[get(copied)]
    pub(crate) peak: usize,
}

impl MemoryStats {
    #[inline]
    #[doc(hidden)]
    pub fn update(&mut self) {
        self.allocated = allocated_bytes();
        self.deallocated = deallocated_bytes();
        self.current = current_bytes();
        self.peak = peak_bytes();
    }
}
