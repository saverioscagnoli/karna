use std::{
    fmt,
    ops::{Add, AddAssign, Div, Mul, Sub, SubAssign},
};

/// This struct shall be used only when writing logic
/// that requires the use of a rectangle, but that must not
/// be user-facing. This rect is internal only.
///
/// So that the user can bring in their own rectangle abstractions
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
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
