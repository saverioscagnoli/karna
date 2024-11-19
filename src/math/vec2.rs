use crate::traits::ToF32;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, Debug)]
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
        Self::new(0, 0)
    }

    pub fn one() -> Self {
        Self::new(1, 1)
    }

    pub fn up() -> Self {
        Self::new(0, -1)
    }

    pub fn down() -> Self {
        Self::new(0, 1)
    }

    pub fn left() -> Self {
        Self::new(-1, 0)
    }

    pub fn right() -> Self {
        Self::new(1, 0)
    }

    pub fn set<F: ToF32>(&mut self, x: F, y: F) {
        self.x = x.to_f32();
        self.y = y.to_f32();
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let length = self.length();

        if length == 0.0 {
            return Self::zero();
        }

        Self::new(self.x / length, self.y / length)
    }

    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn angle(&self, other: Self) -> f32 {
        let dot = self.dot(other);
        let length = self.length() * other.length();

        if length == 0.0 {
            return 0.0;
        }

        (dot / length).acos()
    }

    pub fn distance(&self, other: Self) -> f32 {
        let x = self.x - other.x;
        let y = self.y - other.y;

        (x * x + y * y).sqrt()
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
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<F: ToF32> Add<F> for Vec2 {
    type Output = Self;

    fn add(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();
        Self::new(self.x + rhs, self.y + rhs)
    }
}

impl Add<Vec2> for f32 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self + rhs.x, self + rhs.y)
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

impl AddAssign<Vec2> for f32 {
    fn add_assign(&mut self, rhs: Vec2) {
        *self += rhs.x;
        *self += rhs.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<F: ToF32> Sub<F> for Vec2 {
    type Output = Self;

    fn sub(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();
        Self::new(self.x - rhs, self.y - rhs)
    }
}

impl Sub<Vec2> for f32 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self - rhs.x, self - rhs.y)
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

impl SubAssign<Vec2> for f32 {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self -= rhs.x;
        *self -= rhs.y;
    }
}

impl Mul for Vec2 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl<F: ToF32> Mul<F> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self * rhs.x, self * rhs.y)
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

impl MulAssign<Vec2> for f32 {
    fn mul_assign(&mut self, rhs: Vec2) {
        *self *= rhs.x;
        *self *= rhs.y;
    }
}

impl Div for Vec2 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl<F: ToF32> Div<F> for Vec2 {
    type Output = Self;

    fn div(self, rhs: F) -> Self::Output {
        let rhs = rhs.to_f32();
        Self::new(self.x / rhs, self.y / rhs)
    }
}

impl Div<Vec2> for f32 {
    type Output = Vec2;

    fn div(self, rhs: Vec2) -> Self::Output {
        Vec2::new(self / rhs.x, self / rhs.y)
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

impl DivAssign<Vec2> for f32 {
    fn div_assign(&mut self, rhs: Vec2) {
        *self /= rhs.x;
        *self /= rhs.y;
    }
}
