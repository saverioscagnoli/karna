use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use sdl2::rect::FPoint;

use super::ToF32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    /// Creates a new bidimensional vector.
    /// Can be used in various ways, such as representing the position of an object on a 2D plane,
    /// the velocity of an object, etc.
    ///
    /// # Arguments
    ///
    /// * `x` - The x component of the vector.
    /// * `y` - The y component of the vector.
    pub fn new<T>(x: T, y: T) -> Self
    where
        T: ToF32,
    {
        Self {
            x: x.to_f32(),
            y: y.to_f32(),
        }
    }

    /// Sets the x and y components of the vector.
    ///
    /// # Arguments
    ///
    /// * `x` - The new x component of the vector.
    /// * `y` - The new y component of the vector.
    pub fn set<T>(&mut self, x: T, y: T)
    where
        T: ToF32,
    {
        self.x = x.to_f32();
        self.y = y.to_f32();
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let length = self.length();

        Self {
            x: self.x / length,
            y: self.y / length,
        }
    }
}

impl<T> From<(T, T)> for Vector2
where
    T: ToF32,
{
    fn from((x, y): (T, T)) -> Self {
        Self::new(x, y)
    }
}

impl Into<(f32, f32)> for Vector2 {
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl Into<FPoint> for Vector2 {
    fn into(self) -> FPoint {
        FPoint::new(self.x, self.y)
    }
}

impl Add<Vector2> for Vector2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> Add<T> for Vector2
where
    T: ToF32,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self {
        Self {
            x: self.x + rhs.to_f32(),
            y: self.y + rhs.to_f32(),
        }
    }
}

impl AddAssign<Vector2> for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub<Vector2> for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> Sub<T> for Vector2
where
    T: ToF32,
{
    type Output = Self;

    fn sub(self, rhs: T) -> Self {
        Self {
            x: self.x - rhs.to_f32(),
            y: self.y - rhs.to_f32(),
        }
    }
}

impl SubAssign<Vector2> for Vector2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<Vector2> for Vector2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T> Mul<T> for Vector2
where
    T: ToF32,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self {
        Self {
            x: self.x * rhs.to_f32(),
            y: self.y * rhs.to_f32(),
        }
    }
}

impl MulAssign<Vector2> for Vector2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<Vector2> for Vector2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<T> Div<T> for Vector2
where
    T: ToF32,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self {
        Self {
            x: self.x / rhs.to_f32(),
            y: self.y / rhs.to_f32(),
        }
    }
}

impl DivAssign<Vector2> for Vector2 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}
