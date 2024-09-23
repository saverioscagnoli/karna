use std::{
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
    time::Duration,
};

pub trait ToF32 {
    fn to_f32(&self) -> f32;
}

impl ToF32 for f32 {
    fn to_f32(&self) -> f32 {
        *self
    }
}

impl ToF32 for f64 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for i32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for i64 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for u32 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for u64 {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for usize {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

impl ToF32 for isize {
    fn to_f32(&self) -> f32 {
        *self as f32
    }
}

pub enum Easing {
    Linear,
    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::QuadraticIn => t * t,
            Self::QuadraticOut => t * (2.0 - t),
            Self::QuadraticInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
            Self::CubicIn => t * t * t,
            Self::CubicOut => (t - 1.0).powi(3) + 1.0,
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    (t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0
                }
            }
            Self::BounceIn => 1.0 - Self::BounceOut.apply(1.0 - t),
            Self::BounceOut => {
                if t < 4.0 / 11.0 {
                    (121.0 * t * t) / 16.0
                } else if t < 8.0 / 11.0 {
                    (363.0 / 40.0 * t * t) - (99.0 / 10.0 * t) + 17.0 / 5.0
                } else if t < 9.0 / 10.0 {
                    (4356.0 / 361.0 * t * t) - (35442.0 / 1805.0 * t) + 16061.0 / 1805.0
                } else {
                    (54.0 / 5.0 * t * t) - (513.0 / 25.0 * t) + 268.0 / 25.0
                }
            }
            Self::BounceInOut => {
                if t < 0.5 {
                    0.5 * Self::BounceIn.apply(t * 2.0)
                } else {
                    0.5 * Self::BounceOut.apply(t * 2.0 - 1.0) + 0.5
                }
            }
        }
    }
}

pub fn lerp<T>(start: T, end: T, t: f32) -> T
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<f32, Output = T> + Copy,
{
    start + (end - start) * t
}

pub trait Tweenable {
    fn tween(&self, target: Self, t: f32) -> Self;
}

impl Tweenable for f32 {
    fn tween(&self, target: Self, t: f32) -> Self {
        lerp(*self, target, t)
    }
}

impl Tweenable for Vector2 {
    fn tween(&self, target: Self, t: f32) -> Self {
        Self {
            x: self.x.tween(target.x, t),
            y: self.y.tween(target.y, t),
        }
    }
}

pub struct Tween<T> {
    start: T,
    target: T,
    duration: f32,
    elapsed: f32,
    easing: Easing,
}

impl<T> Tween<T>
where
    T: Tweenable + Copy,
{
    pub fn new(start: T, target: T, duration: Duration, easing: Easing) -> Self {
        Self {
            start,
            target,
            duration: duration.as_secs_f32(),
            elapsed: 0.0,
            easing,
        }
    }

    pub fn update(&mut self, dt: f32) -> T {
        self.elapsed += dt;

        if self.elapsed >= self.duration {
            return self.target;
        }

        self.start
            .tween(self.target, self.easing.apply(self.elapsed / self.duration))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: impl ToF32, y: impl ToF32) -> Self {
        Self {
            x: x.to_f32(),
            y: y.to_f32(),
        }
    }

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn set(&mut self, x: impl ToF32, y: impl ToF32) {
        self.x = x.to_f32();
        self.y = y.to_f32();
    }
}

impl<T> From<(T, T)> for Vector2
where
    T: ToF32,
{
    fn from((x, y): (T, T)) -> Self {
        Self {
            x: x.to_f32(),
            y: y.to_f32(),
        }
    }
}

impl Add for Vector2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
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

    fn add(self, rhs: T) -> Self::Output {
        Self {
            x: self.x + rhs.to_f32(),
            y: self.y + rhs.to_f32(),
        }
    }
}

impl AddAssign for Vector2 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Sub for Vector2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
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

    fn sub(self, rhs: T) -> Self::Output {
        Self {
            x: self.x - rhs.to_f32(),
            y: self.y - rhs.to_f32(),
        }
    }
}

impl SubAssign for Vector2 {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Mul<f32> for Vector2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl MulAssign<f32> for Vector2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

impl<T> From<(T, T)> for Size
where
    T: ToF32,
{
    fn from((width, height): (T, T)) -> Self {
        Self {
            width: width.to_f32(),
            height: height.to_f32(),
        }
    }
}
