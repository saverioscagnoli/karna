use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Sub, SubAssign};

use super::ToF32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    /// (x: 0.0, y: 0.0, z: 0.0)
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    /// (x: 1.0, y: 1.0, z: 1.0)
    pub const ONE: Self = Self {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    /// (x: 0.0, y: 1.0, z: 0.0)
    pub const UP: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    /// (x: 0.0, y: -1.0, z: 0.0)
    pub const DOWN: Self = Self {
        x: 0.0,
        y: -1.0,
        z: 0.0,
    };
    /// (x: -1.0, y: 0.0, z: 0.0)
    pub const LEFT: Self = Self {
        x: -1.0,
        y: 0.0,
        z: 0.0,
    };
    /// (x: 1.0, y: 0.0, z: 0.0)
    pub const RIGHT: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    /// (x: 0.0, y: 0.0, z: 1.0)
    pub const FORWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    /// (x: 0.0, y: 0.0, z: -1.0)
    pub const BACKWARD: Self = Self {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    /// Creates a new tridimensional vector
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn as_ptr(&self) -> *const f32 {
        &self.x as *const f32
    }

    /// Sets the x, y, and z values of the vector at the same time
    pub fn set(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    /// Returns the length of the vector.
    /// Uses the pythagorean theorem
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Normalizes the vector
    /// (makes the length of the vector 1)
    pub fn normalize(&mut self) {
        let length = self.length();
        self.x /= length;
        self.y /= length;
        self.z /= length;
    }

    /// Returns the dot product of two vectors
    /// # Examples
    /// ```
    /// let a = Vec3::new(1.0, 2.0, 3.0);
    /// let b = Vec3::new(4.0, 5.0, 6.0);
    /// let dot = a.dot(&b);
    ///
    /// assert_eq!(32.0, dot);
    /// ```
    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Returns the cross product of two vectors
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Returns the angle between two vectors
    pub fn angle(&self, other: &Self) -> f32 {
        let dot = self.dot(other);
        let length1 = self.length();
        let length2 = other.length();
        let cos_theta = dot / (length1 * length2);
        cos_theta.acos()
    }

    /// Returns the distance between two vectors
    pub fn distance(&self, other: &Self) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        let z = self.z - other.z;
        (x * x + y * y + z * z).sqrt()
    }

    /// Returns the squared distance between two vectors
    /// Avoids the square root operation
    pub fn distance_squared(&self, other: &Self) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        let z = self.z - other.z;
        x * x + y * y + z * z
    }

    /// Linearly interpolates between two vectors
    /// Useful for smooth transitions between two points
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds"),
        }
    }
}

impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            _ => panic!("Index out of bounds"),
        }
    }
}

/// Creates a Vec3 from a tuple
/// # Examples
/// ```
/// let a = (1.0, 2.0, 3.0);
/// let b = Vec3::from(a);
///
/// assert_eq!(b, Vec3::new(1.0, 2.0, 3.0));
/// ```
impl<F: ToF32> From<(F, F, F)> for Vec3 {
    fn from(tuple: (F, F, F)) -> Self {
        Self {
            x: tuple.0.to_f32(),
            y: tuple.1.to_f32(),
            z: tuple.2.to_f32(),
        }
    }
}

/// Creates a tuple from a Vec3
/// # Examples
/// ```
/// let a = Vec3::new(1.0, 2.0, 3.0);
/// let b = (1.0, 2.0, 3.0);
///
/// assert_eq!(a.into(), b);
/// ```
impl From<Vec3> for (f32, f32, f32) {
    fn from(vec: Vec3) -> Self {
        (vec.x, vec.y, vec.z)
    }
}

/// Vector addition
/// # Examples
/// ```
/// let a = Vec3::new(1.0, 2.0, 3.0);
/// let b = Vec3::new(4.0, 5.0, 6.0);
///
/// assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
/// ```
impl Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

/// Vector addition assignment
/// # Examples
/// ```
/// let mut a = Vec3::new(1.0, 2.0, 3.0);
/// let b = Vec3::new(4.0, 5.0, 6.0);
///
/// a += b;
///
/// assert_eq!(a, Vec3::new(5.0, 7.0, 9.0));
/// ```
impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        };
    }
}

/// Scalar addition
/// # Examples
/// ```
/// let a = Vec3::new(1.0, 2.0, 3.0);
/// let b = 4.0;
///
/// assert_eq!(a + b, Vec3::new(5.0, 6.0, 7.0));
/// ```
impl Add<f32> for Vec3 {
    type Output = Self;

    fn add(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        }
    }
}

/// Scalar addition assignment
/// # Examples
/// ```
/// let mut a = Vec3::new(1.0, 2.0, 3.0);
/// let b = 4.0;
///
/// a += b;
///
/// assert_eq!(a, Vec3::new(5.0, 6.0, 7.0));
/// ```
impl AddAssign<f32> for Vec3 {
    fn add_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
        };
    }
}

/// Vector subtraction
/// # Examples
/// ```
/// let a = Vec3::new(5.0, 7.0, 9.0);
/// let b = Vec3::new(1.0, 2.0, 3.0);
///
/// assert_eq!(a - b, Vec3::new(4.0, 5.0, 6.0));
/// ```
impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

/// Vector subtraction assignment
/// # Examples
/// ```
/// let mut a = Vec3::new(5.0, 7.0, 9.0);
/// let b = Vec3::new(1.0, 2.0, 3.0);
///
/// a -= b;
///
/// assert_eq!(a, Vec3::new(4.0, 5.0, 6.0));
/// ```
impl SubAssign for Vec3 {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        };
    }
}

/// Scalar subtraction
/// # Examples
/// ```
/// let a = Vec3::new(5.0, 7.0, 9.0);
/// let b = 1.0;
///
/// assert_eq!(a - b, Vec3::new(4.0, 6.0, 8.0));
/// ```
impl Sub<f32> for Vec3 {
    type Output = Self;

    fn sub(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

/// Scalar subtraction assignment
/// # Examples
/// ```
/// let mut a = Vec3::new(5.0, 7.0, 9.0);
/// let b = 1.0;
///
/// a -= b;
///
/// assert_eq!(a, Vec3::new(4.0, 6.0, 8.0));
/// ```
impl SubAssign<f32> for Vec3 {
    fn sub_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        };
    }
}

/// Vector multiplication
/// # Examples
/// ```
/// let a = Vec3::new(1.0, 2.0, 3.0);
/// let b = Vec3::new(4.0, 5.0, 6.0);
///
/// assert_eq!(a * b, Vec3::new(4.0, 10.0, 18.0));
/// ```
impl Mul for Vec3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

/// Vector multiplication assignment
/// # Examples
/// ```
/// let mut a = Vec3::new(1.0, 2.0, 3.0);
/// let b = Vec3::new(4.0, 5.0, 6.0);
///
/// a *= b;
///
/// assert_eq!(a, Vec3::new(4.0, 10.0, 18.0));
/// ```
impl MulAssign for Vec3 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        };
    }
}

/// Scalar multiplication
/// # Examples
/// ```
/// let a = Vec3::new(1.0, 2.0, 3.0);
/// let b = 4.0;
///
/// assert_eq!(a * b, Vec3::new(4.0, 8.0, 12.0));
/// ```
impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

/// Scalar multiplication assignment
/// # Examples
/// ```
/// let mut a = Vec3::new(1.0, 2.0, 3.0);
/// let b = 4.0;
///
/// a *= b;
///
/// assert_eq!(a, Vec3::new(4.0, 8.0, 12.0));
/// ```
impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, rhs: f32) {
        *self = Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        };
    }
}

/// Vector division
/// # Examples
/// ```
/// let a = Vec3::new(4.0, 9.0, 16.0);
/// let b = Vec3::new(2.0, 3.0, 4.0);
///
/// assert_eq!(a / b, Vec3::new(2.0, 3.0, 4.0));
/// ```
/// # Panics
/// Panics if any component of the divisor is zero
impl Div for Vec3 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.x == 0.0 || rhs.y == 0.0 || rhs.z == 0.0 {
            panic!("Division by zero");
        }

        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

/// Vector division assignment
/// # Examples
/// ```
/// let mut a = Vec3::new(4.0, 9.0, 16.0);
/// let b = Vec3::new(2.0, 3.0, 4.0);
///
/// a /= b;
///
/// assert_eq!(a, Vec3::new(2.0, 3.0, 4.0));
/// ```
/// # Panics
/// Panics if any component of the divisor is zero
impl DivAssign for Vec3 {
    fn div_assign(&mut self, rhs: Self) {
        if rhs.x == 0.0 || rhs.y == 0.0 || rhs.z == 0.0 {
            panic!("Division by zero");
        }

        *self = Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        };
    }
}

/// Scalar division
/// # Examples
/// ```
/// let a = Vec3::new(4.0, 9.0, 16.0);
/// let b = 2.0;
///
/// assert_eq!(a / b, Vec3::new(2.0, 4.5, 8.0));
/// ```
/// # Panics
/// Panics if the divisor is zero
impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        if rhs == 0.0 {
            panic!("Division by zero");
        }

        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

/// Scalar division assignment
/// # Examples
/// ```
/// let mut a = Vec3::new(4.0, 9.0, 16.0);
/// let b = 2.0;
///
/// a /= b;
///
/// assert_eq!(a, Vec3::new(2.0, 4.5, 8.0));
/// ```
/// # Panics
/// Panics if the divisor is zero
impl DivAssign<f32> for Vec3 {
    fn div_assign(&mut self, rhs: f32) {
        if rhs == 0.0 {
            panic!("Division by zero");
        }

        *self = Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        };
    }
}

/// Reverse scalar addition
/// # Examples
/// ```
/// let a = 4.0;
/// let b = Vec3::new(1.0, 2.0, 3.0);
///
/// assert_eq!(a + b, Vec3::new(5.0, 6.0, 7.0));
/// ```
impl Add<Vec3> for f32 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self + rhs.x,
            y: self + rhs.y,
            z: self + rhs.z,
        }
    }
}

/// Reverse scalar subtraction
/// # Examples
/// ```
/// let a = 4.0;
/// let b = Vec3::new(1.0, 2.0, 3.0);
///
/// assert_eq!(a - b, Vec3::new(3.0, 2.0, 1.0));
/// ```
impl Sub<Vec3> for f32 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self - rhs.x,
            y: self - rhs.y,
            z: self - rhs.z,
        }
    }
}

/// Reverse scalar multiplication
/// # Examples
/// ```
/// let a = 4.0;
/// let b = Vec3::new(1.0, 2.0, 3.0);
///
/// assert_eq!(a * b, Vec3::new(4.0, 8.0, 12.0));
/// ```
impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

/// Reverse scalar division
/// # Examples
/// ```
/// let a = 8.0;
/// let b = Vec3::new(2.0, 4.0, 8.0);
///
/// assert_eq!(a / b, Vec3::new(4.0, 2.0, 1.0));
/// ```
impl Div<Vec3> for f32 {
    type Output = Vec3;

    fn div(self, rhs: Vec3) -> Self::Output {
        Vec3 {
            x: self / rhs.x,
            y: self / rhs.y,
            z: self / rhs.z,
        }
    }
}
