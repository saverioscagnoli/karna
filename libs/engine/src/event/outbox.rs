use nostd::alloc::vec::Vec;
use utils::Label;

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
    pub fn drain_into(&mut self, out: &mut Vec<T>) {
        if self.buf.len() > self.peak {
            self.peak = self.buf.len()
        }

        out.append(&mut self.buf);
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
}
