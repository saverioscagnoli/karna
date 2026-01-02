mod label_map;
mod slot_map;

use std::{
    fmt,
    ops::{Add, AddAssign, Deref, DerefMut, Div, Mul, Sub, SubAssign},
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

/// A type representing a size in bytes, similar to std::time::Duration.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct ByteSize(u64);

impl ByteSize {
    /// The zero byte size.
    pub const ZERO: Self = Self(0);

    /// The maximum representable byte size.
    pub const MAX: Self = Self(u64::MAX);

    // Constructors

    /// Creates a new `ByteSize` from the specified number of bytes.
    pub const fn from_bytes(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Creates a new `ByteSize` from the specified number of kilobytes (1 KB = 1024 bytes).
    pub const fn from_kb(kb: u64) -> Self {
        Self(kb * 1024)
    }

    /// Creates a new `ByteSize` from the specified number of megabytes (1 MB = 1024² bytes).
    pub const fn from_mb(mb: u64) -> Self {
        Self(mb * 1024 * 1024)
    }

    /// Creates a new `ByteSize` from the specified number of gigabytes (1 GB = 1024³ bytes).
    pub const fn from_gb(gb: u64) -> Self {
        Self(gb * 1024 * 1024 * 1024)
    }

    /// Creates a new `ByteSize` from the specified number of terabytes (1 TB = 1024⁴ bytes).
    pub const fn from_tb(tb: u64) -> Self {
        // Check for overflow
        if let Some(bytes) = tb.checked_mul(1024 * 1024 * 1024 * 1024) {
            Self(bytes)
        } else {
            Self(u64::MAX)
        }
    }

    /// Returns the total number of bytes.
    pub const fn as_bytes(&self) -> u64 {
        self.0
    }

    /// Returns the total number of kilobytes.
    pub const fn as_kb(&self) -> u64 {
        self.0 / 1024
    }

    /// Returns the total number of megabytes.
    pub const fn as_mb(&self) -> u64 {
        self.0 / (1024 * 1024)
    }

    /// Returns the total number of gigabytes.
    pub const fn as_gb(&self) -> u64 {
        self.0 / (1024 * 1024 * 1024)
    }

    /// Returns the total number of terabytes.
    pub const fn as_tb(&self) -> u64 {
        self.0 / (1024 * 1024 * 1024 * 1024)
    }

    /// Returns the total number of kilobytes as a floating point.
    pub fn as_kb_f64(&self) -> f64 {
        self.0 as f64 / 1024.0
    }

    /// Returns the total number of megabytes as a floating point.
    pub fn as_mb_f64(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0)
    }

    /// Returns the total number of gigabytes as a floating point.
    pub fn as_gb_f64(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Returns the total number of terabytes as a floating point.
    pub fn as_tb_f64(&self) -> f64 {
        self.0 as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)
    }

    // Checked operations

    /// Checked addition. Returns `None` if overflow occurred.
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        if let Some(bytes) = self.0.checked_add(rhs.0) {
            Some(Self(bytes))
        } else {
            None
        }
    }

    /// Checked subtraction. Returns `None` if overflow occurred.
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        if let Some(bytes) = self.0.checked_sub(rhs.0) {
            Some(Self(bytes))
        } else {
            None
        }
    }

    /// Checked multiplication by a scalar. Returns `None` if overflow occurred.
    pub const fn checked_mul(self, rhs: u64) -> Option<Self> {
        if let Some(bytes) = self.0.checked_mul(rhs) {
            Some(Self(bytes))
        } else {
            None
        }
    }

    /// Checked division by a scalar. Returns `None` if `rhs` is 0.
    pub const fn checked_div(self, rhs: u64) -> Option<Self> {
        if let Some(bytes) = self.0.checked_div(rhs) {
            Some(Self(bytes))
        } else {
            None
        }
    }

    /// Saturating addition.
    pub const fn saturating_add(self, rhs: Self) -> Self {
        Self(self.0.saturating_add(rhs.0))
    }

    /// Saturating subtraction.
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Add for ByteSize {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for ByteSize {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for ByteSize {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for ByteSize {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul<u64> for ByteSize {
    type Output = Self;

    fn mul(self, rhs: u64) -> Self {
        Self(self.0 * rhs)
    }
}

impl Div<u64> for ByteSize {
    type Output = Self;

    fn div(self, rhs: u64) -> Self {
        Self(self.0 / rhs)
    }
}

/// Display implementation with human-readable formatting
impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;
        const TB: f64 = GB * 1024.0;

        let bytes = self.0 as f64;

        if bytes >= TB {
            write!(f, "{:.2} TB", bytes / TB)
        } else if bytes >= GB {
            write!(f, "{:.2} GB", bytes / GB)
        } else if bytes >= MB {
            write!(f, "{:.2} MB", bytes / MB)
        } else if bytes >= KB {
            write!(f, "{:.2} KB", bytes / KB)
        } else {
            write!(f, "{} B", self.0)
        }
    }
}
