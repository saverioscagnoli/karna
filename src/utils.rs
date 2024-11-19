use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicU32, Ordering},
};

#[derive(Debug)]
pub struct AtomicF32 {
    inner: UnsafeCell<f32>,
}

impl AtomicF32 {
    pub const fn new(value: f32) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }

    fn as_atomic_bits(&self) -> &AtomicU32 {
        // Safety: All potentially shared reads/writes go through this, and the
        // static assertions above ensure that AtomicU32 and UnsafeCell<f32> are
        // compatible as pointers.
        unsafe { &*(&self.inner as *const _ as *const AtomicU32) }
    }

    pub fn load(&self, order: Ordering) -> f32 {
        f32::from_bits(self.as_atomic_bits().load(order))
    }

    pub fn store(&self, value: f32, order: Ordering) {
        self.as_atomic_bits().store(value.to_bits(), order);
    }

    pub fn swap(&self, value: f32, order: Ordering) -> f32 {
        f32::from_bits(self.as_atomic_bits().swap(value.to_bits(), order))
    }
}

unsafe impl Send for AtomicF32 {}
unsafe impl Sync for AtomicF32 {}
