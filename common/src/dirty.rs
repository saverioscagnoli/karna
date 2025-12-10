use std::cell::Cell;
use std::ops::{Deref, DerefMut};

pub struct DirtyTracked<T> {
    value: T,
    dirty: Cell<bool>,
}

impl<T> From<T> for DirtyTracked<T> {
    fn from(value: T) -> Self {
        Self {
            value,
            dirty: Cell::new(true),
        }
    }
}

impl<T> DirtyTracked<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            dirty: Cell::new(true),
        }
    }

    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    #[inline]
    pub fn mark(&self) {
        self.dirty.set(true);
    }

    #[inline]
    pub fn clean(&self) {
        self.dirty.set(false);
    }
}

impl<T> Deref for DirtyTracked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for DirtyTracked<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.dirty.set(true);
        &mut self.value
    }
}
