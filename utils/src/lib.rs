mod label_map;
mod slot_map;

use std::{
    ops::{Deref, DerefMut},
    time::Duration,
};

// Re-exports
pub use label_map::*;
pub use slot_map::*;

/// Implement Deref + DerefMut by transmuting from one type to another.
///
/// IMPORTANT: Both types MUST have the same memory layout!!
#[macro_export]
macro_rules! impl_deref_to {
    ($from:ty => $to:ty) => {
        impl ::std::ops::Deref for $from {
            type Target = $to;

            #[inline]
            fn deref(&self) -> &Self::Target {
                unsafe { &*(self as *const Self as *const $to) }
            }
        }

        impl ::std::ops::DerefMut for $from {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe { &mut *(self as *mut Self as *mut $to) }
            }
        }
    };
}

pub fn as_u8_slice<T: Sized>(slice: &[T]) -> &[u8] {
    unsafe {
        ::core::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * ::core::mem::size_of::<T>(),
        )
    }
}

#[derive(Debug)]
pub struct Lazy<T>(Option<T>);

impl<T> Lazy<T> {
    #[inline]
    pub fn new() -> Self {
        Self(None)
    }

    #[inline]
    pub fn set(&mut self, value: T) {
        self.0 = Some(value)
    }

    #[inline]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

impl<T> Deref for Lazy<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().expect("Not initialized")
    }
}

impl<T> DerefMut for Lazy<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().expect("Not initialized")
    }
}

pub struct Timer {
    elapsed: f32,
    duration: f32,
    paused: bool,
}

impl Timer {
    /// Creates a new timer with a set duration in seconds.
    pub fn new(duration: Duration) -> Self {
        Self {
            elapsed: 0.0,
            duration: duration.as_secs_f32(),
            paused: false,
        }
    }

    /// Updates the timer. Call this once per frame with the delta time.
    pub fn tick(&mut self, dt: f32) {
        if !self.paused && !self.is_finished() {
            self.elapsed += dt;
        }
    }

    /// Returns true only on the frame where the timer finishes.
    /// Useful for one-time events.
    pub fn just_finished(&self, dt: f32) -> bool {
        self.elapsed >= self.duration && self.elapsed - dt < self.duration
    }

    /// Returns true once the timer has finished (stays true).
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Returns the progress as a value between 0.0 and 1.0.
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Resets the timer to start from 0 again.
    #[inline]
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Pauses the timer.
    #[inline]
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the timer.
    #[inline]
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Remaining time in seconds.
    #[inline]
    pub fn remaining(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0)
    }
}
