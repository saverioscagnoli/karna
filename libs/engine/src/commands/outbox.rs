use core::mem;

use nostd::alloc::vec::Vec;
use nostd::alloc::vec::{self};

pub struct Outbox<T> {
    buf: Vec<T>,
    cap: usize,
    name: &'static str,
    peak: usize,
    dropped: u32,
}

impl<T> Outbox<T> {
    pub fn new(name: &'static str, cap: usize) -> Self {
        Self {
            buf: Vec::with_capacity(cap),
            cap,
            name,
            peak: 0,
            dropped: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, event: T) {
        if self.buf.len() >= self.cap {
            debug_assert!(false, "Outbox '{}' overflowed: cap {}", self.name, self.cap);
            self.dropped = self.dropped.saturating_add(1);
            return;
        }

        self.buf.push(event);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    #[inline]
    pub fn cap(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn drain(&mut self) -> vec::Drain<'_, T> {
        self.peak = self.peak.max(self.buf.len());
        self.buf.drain(..)
    }

    #[inline]
    pub fn take(&mut self) -> Vec<T> {
        self.peak = self.peak.max(self.buf.len());
        mem::take(&mut self.buf)
    }

    #[inline]
    pub fn restore(&mut self, mut buf: Vec<T>) {
        buf.clear();
        buf.append(&mut self.buf);
        self.buf = buf;
    }
}
