use crate::traits::ToF32;

use super::Vec2;

use std::{
    ops::{Add, Mul, Sub},
    time::Duration,
};

pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy,
{
    a + (b - a) * t
}

pub trait Interpolate {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        lerp(*self, *other, t)
    }
}

impl Interpolate for u32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + ((*other as f32 - *self as f32) * t) as u32
    }
}

impl Interpolate for Vec2 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Vec2 {
            x: lerp(self.x, other.x, t),
            y: lerp(self.y, other.y, t),
        }
    }
}

#[derive(Clone, Copy, Debug)]
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

pub struct Tween<T: Interpolate> {
    start: T,
    target: T,
    duration: f32,
    elapsed: f32,
    easing: Easing,
}

impl<T: Interpolate + Copy> Tween<T> {
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

        self.start.interpolate(
            &self.target,
            self.easing.apply(self.elapsed / self.duration),
        )
    }

    pub fn is_over(&self) -> bool {
        self.elapsed >= self.duration
    }

    pub fn reversed(&self) -> Self {
        Self {
            start: self.target,
            target: self.start,
            duration: self.duration,
            elapsed: 0.0,
            easing: self.easing,
        }
    }
}
