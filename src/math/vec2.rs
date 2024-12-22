use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use sdl2::rect::FPoint;

use super::ToF32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// (x: 0.0, y: 0.0)
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };
    /// (x: 1.0, y: 1.0)
    pub const ONE: Self = Self { x: 1.0, y: 1.0 };
    /// (x: 1.0, y: 0.0)
    pub const RIGHT: Self = Self { x: 1.0, y: 0.0 };
    /// (x: 0.0, y: 1.0)
    pub const UP: Self = Self { x: 0.0, y: 1.0 };
    /// (x: -1.0, y: 0.0)
    pub const DOWN: Self = Self { x: 0.0, y: -1.0 };
    /// (x: 0.0, y: -1.0)
    pub const LEFT: Self = Self { x: -1.0, y: 0.0 };

    /// Creates a new bidimensional vector
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn as_ptr(&self) -> *const f32 {
        &self.x as *const f32
    }

    /// Set x and y values at the same time
    pub fn set(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    /// Returns the length of the vector
    /// Uses the pythagorean theorem
    pub fn length(&self) -> f32 {
        self.x.hypot(self.y)
    }

    /// Normalizes the vector
    /// (Makes the length of the vector 1)
    pub fn normalize(&self) -> Self {
        let length = self.length();

        // Avoid division by zero
        if length == 0.0 {
            return Self::ZERO;
        }

        Self {
            x: self.x / length,
            y: self.y / length,
        }
    }

    /// Returns the dot product of two vectors
    /// # Examples
    /// ```rust
    /// let a = Vec2::new(1.0, 2.0);
    /// let b = Vec2::new(3.0, 4.0);
    ///
    /// assert_eq!(a.dot(b), 11.0);
    /// ```
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    /// Returns the angle between two vectors
    pub fn angle(&self, other: Self) -> f32 {
        let dot = self.dot(other);
        let lengths = self.length() * other.length();

        if lengths == 0.0 {
            return 0.0;
        }

        (dot / lengths).acos()
    }

    /// Returns the distance between two vectors
    pub fn distance(&self, other: Self) -> f32 {
        (*self - other).length()
    }

    /// Returns the squared distance between two vectors
    /// Avoids the square root operation
    pub fn distance_squared(&self, other: Self) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;

        x * x + y * y
    }

    /// Linear interpolation between two vectors
    /// Useful for smooth transitions between two points
    pub fn lerp(&self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl From<Vec2> for FPoint {
    fn from(vec: Vec2) -> Self {
        Self::new(vec.x, vec.y)
    }
}

/// Converts a tuple into a Vec2
/// # Examples
/// ```rust
/// let tuple = (1.0, 2.0);
/// let vec = Vec2::from(tuple);
/// assert_eq!(vec, Vec2::new(1.0, 2.0));
/// ```
impl<F: ToF32> From<(F, F)> for Vec2 {
    fn from(tuple: (F, F)) -> Self {
        Self {
            x: tuple.0.to_f32(),
            y: tuple.1.to_f32(),
        }
    }
}

/// Converts a Vec2 into a tuple
/// # Examples
/// ```rust
/// let vec = Vec2::new(1.0, 2.0);
/// let tuple: (f32, f32) = vec.into();
/// assert_eq!(tuple, (1.0, 2.0));
/// ```
impl From<Vec2> for (f32, f32) {
    fn from(vec: Vec2) -> Self {
        (vec.x, vec.y)
    }
}

/// Vector addition
/// # Examples
/// ```
/// let a = Vec2::new(1.0, 2.0);
/// let b = Vec2::new(3.0, 4.0);
///
/// assert_eq!(a + b, Vec2::new(4.0, 6.0));
/// ```
impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// Vector addition assignment
/// # Examples
/// ```
/// let mut a = Vec2::new(1.0, 2.0);
/// let b = Vec2::new(3.0, 4.0);
///
/// a += b;
///
/// assert_eq!(a, Vec2::new(4.0, 6.0));
/// ```
impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        };
    }
}

/// Float addition (Adds a float to both x and y)
/// # Examples
/// ```
/// let a = Vec2::new(1.0, 2.0);
/// let b = 3.0;
///
/// assert_eq!(a + b, Vec2::new(4.0, 5.0));
/// ```
impl Add<f32> for Vec2 {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

/// Float addition assignment (Adds a float to both x and y)
/// # Examples
/// ```
/// let mut a = Vec2::new(1.0, 2.0);
/// let b = 3.0;
///
/// a += b;
///
/// assert_eq!(a, Vec2::new(4.0, 5.0));
/// ```
impl AddAssign<f32> for Vec2 {
    fn add_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x + rhs,
            y: self.y + rhs,
        };
    }
}

/// Vector subtraction
/// # Examples
/// ```
/// let a = Vec2::new(5.0, 6.0);
/// let b = Vec2::new(3.0, 4.0);
///
/// assert_eq!(a - b, Vec2::new(2.0, 2.0));
/// ```
impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// Vector subtraction assignment
/// # Examples
/// ```
/// let mut a = Vec2::new(5.0, 6.0);
/// let b = Vec2::new(3.0, 4.0);
///
/// a -= b;
///
/// assert_eq!(a, Vec2::new(2.0, 2.0));
/// ```
impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        };
    }
}

/// Float subtraction (Subtracts a float from both x and y)
/// # Examples
/// ```
/// let a = Vec2::new(5.0, 6.0);
/// let b = 3.0;
///
/// assert_eq!(a - b, Vec2::new(2.0, 3.0));
/// ```
impl Sub<f32> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

/// Float subtraction assignment (Subtracts a float from both x and y)
/// # Examples
/// ```
/// let mut a = Vec2::new(5.0, 6.0);
/// let b = 3.0;
///
/// a -= b;
///
/// assert_eq!(a, Vec2::new(2.0, 3.0));
/// ```
impl SubAssign<f32> for Vec2 {
    fn sub_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x - rhs,
            y: self.y - rhs,
        };
    }
}

/// Vector multiplication
/// # Examples
/// ```
/// let a = Vec2::new(2.0, 3.0);
/// let b = Vec2::new(4.0, 5.0);
///
/// assert_eq!(a * b, Vec2::new(8.0, 15.0));
/// ```
impl Mul for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

/// Vector multiplication assignment
/// # Examples
/// ```
/// let mut a = Vec2::new(2.0, 3.0);
/// let b = Vec2::new(4.0, 5.0);
///
/// a *= b;
///
/// assert_eq!(a, Vec2::new(8.0, 15.0));
/// ```
impl MulAssign for Vec2 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        };
    }
}

/// Scalar multiplication
/// # Examples
/// ```
/// let a = Vec2::new(2.0, 3.0);
/// let b = 4.0;
///
/// assert_eq!(a * b, Vec2::new(8.0, 12.0));
/// ```
impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/// Scalar multiplication assignment
/// # Examples
/// ```
/// let mut a = Vec2::new(2.0, 3.0);
/// let b = 4.0;
///
/// a *= b;
///
/// assert_eq!(a, Vec2::new(8.0, 12.0));
/// ```
impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x * rhs,
            y: self.y * rhs,
        };
    }
}

/// Vector division
/// # Examples
/// ```
/// let a = Vec2::new(8.0, 12.0);
/// let b = Vec2::new(4.0, 3.0);
///
/// assert_eq!(a / b, Vec2::new(2.0, 4.0));
/// ```
/// # Panics
/// Panics if the divisor is zero
impl Div for Vec2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.x == 0.0 || rhs.y == 0.0 {
            panic!("Division by zero");
        }

        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

/// Vector division assignment
/// # Examples
/// ```
/// let mut a = Vec2::new(8.0, 12.0);
/// let b = Vec2::new(4.0, 3.0);
///
/// a /= b;
///
/// assert_eq!(a, Vec2::new(2.0, 4.0));
/// ```
/// # Panics
/// Panics if the divisor is zero
impl DivAssign for Vec2 {
    fn div_assign(&mut self, rhs: Self) {
        if rhs.x == 0.0 || rhs.y == 0.0 {
            panic!("Division by zero");
        }

        *self = Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        };
    }
}

/// Scalar division
/// # Examples
/// ```
/// let a = Vec2::new(8.0, 12.0);
/// let b = 4.0;
///
/// assert_eq!(a / b, Vec2::new(2.0, 3.0));
/// ```
/// # Panics
/// Panics if the divisor is zero
impl Div<f32> for Vec2 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0.0 {
            panic!("Division by zero");
        }

        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

/// Scalar division assignment
/// # Examples
/// ```
/// let mut a = Vec2::new(8.0, 12.0);
/// let b = 4.0;
///
/// a /= b;
///
/// assert_eq!(a, Vec2::new(2.0, 3.0));
/// ```
/// # Panics
/// Panics if the divisor is zero
impl DivAssign<f32> for Vec2 {
    fn div_assign(&mut self, rhs: f32) {
        if rhs == 0.0 {
            panic!("Division by zero");
        }

        *self = Self {
            x: self.x / rhs,
            y: self.y / rhs,
        };
    }
}

/// Reverse float addition
/// # Examples
/// ```
/// let a = 3.0;
/// let b = Vec2::new(1.0, 2.0);
///
/// assert_eq!(a + b, Vec2::new(4.0, 5.0));
/// ```
impl Add<Vec2> for f32 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self + rhs.x,
            y: self + rhs.y,
        }
    }
}

/// Reverse float subtraction
/// # Examples
/// ```
/// let a = 3.0;
/// let b = Vec2::new(1.0, 2.0);
///
/// assert_eq!(a - b, Vec2::new(2.0, 1.0));
/// ```
impl Sub<Vec2> for f32 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self - rhs.x,
            y: self - rhs.y,
        }
    }
}

/// Reverse scalar multiplication
/// # Examples
/// ```
/// let a = 3.0;
/// let b = Vec2::new(1.0, 2.0);
///
/// assert_eq!(a * b, Vec2::new(3.0, 6.0));
/// ```
impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

/// Reverse scalar division
/// # Examples
/// ```
/// let a = 3.0;
/// let b = Vec2::new(1.0, 2.0);
///
/// assert_eq!(a / b, Vec2::new(3.0, 1.5));
/// ```
impl Div<Vec2> for f32 {
    type Output = Vec2;

    fn div(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self / rhs.x,
            y: self / rhs.y,
        }
    }
}
