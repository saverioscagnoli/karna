use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use sdl2::rect::FPoint;

use super::ToF32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new<F: ToF32>(x: F, y: F) -> Self {
        Self {
            x: x.to_f32(),
            y: y.to_f32(),
        }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    pub fn up() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    pub fn down() -> Self {
        Self { x: 0.0, y: -1.0 }
    }

    pub fn left() -> Self {
        Self { x: -1.0, y: 0.0 }
    }

    pub fn right() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    pub fn set<F: ToF32>(&mut self, x: F, y: F) {
        self.x = x.to_f32();
        self.y = y.to_f32();
    }

    pub fn dot(&self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// SDL FPoint
impl From<Vec2> for sdl2::rect::FPoint {
    fn from(vec: Vec2) -> Self {
        FPoint::new(vec.x, vec.y)
    }
}

impl<F: ToF32> From<(F, F)> for Vec2 {
    fn from((x, y): (F, F)) -> Self {
        Self::new(x, y)
    }
}

impl From<Vec2> for (f32, f32) {
    fn from(vec: Vec2) -> Self {
        (vec.x, vec.y)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<F: ToF32> Add<F> for Vec2 {
    type Output = Self;

    fn add(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();

        Self {
            x: self.x + rhs,
            y: self.y + rhs,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<F: ToF32> AddAssign<F> for Vec2 {
    fn add_assign(&mut self, rhs: F) {
        let rhs = rhs.to_f32();

        self.x += rhs;
        self.y += rhs;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<F: ToF32> Sub<F> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();

        Self {
            x: self.x - rhs,
            y: self.y - rhs,
        }
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl<F: ToF32> SubAssign<F> for Vec2 {
    fn sub_assign(&mut self, rhs: F) {
        let rhs = rhs.to_f32();

        self.x -= rhs;
        self.y -= rhs;
    }
}

impl Mul for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<F: ToF32> Mul<F> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();

        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign for Vec2 {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl<F: ToF32> MulAssign<F> for Vec2 {
    fn mul_assign(&mut self, rhs: F) {
        let rhs = rhs.to_f32();

        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div for Vec2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl<F: ToF32> Div<F> for Vec2 {
    type Output = Self;

    fn div(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();

        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl DivAssign for Vec2 {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl<F: ToF32> DivAssign<F> for Vec2 {
    fn div_assign(&mut self, rhs: F) {
        let rhs = rhs.to_f32();

        self.x /= rhs;
        self.y /= rhs;
    }
}
