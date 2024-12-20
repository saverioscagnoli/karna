use sdl2::pixels::Color;

use super::Vec2;
use std::{
    ops::{Add, Mul, Sub},
    time::Duration,
};

/// Linearly interpolates between two values.
/// Useful for creating smooth transitions between values.
///
/// The type `T` must implement the `Add`, `Sub` and `Mul<f32>` traits.
/// basically it must be a type that can be added, subtracted and multiplied by a float.
/// Works with numbers, vectors, colors, etc.
///
/// # Examples
///
/// ```no_run
/// use karna::math::lerp;
///
/// let a = 0.0;
/// let b = 10.0;
///
/// let result = lerp(a, b, 0.5);
///
/// assert_eq!(result, 5.0);
/// ```
pub fn lerp<T>(a: T, b: T, t: f32) -> T
where
    T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy,
{
    a + (b - a) * t
}

/// A trait that allows a type to be interpolated between two values.
/// This is useful for creating smooth transitions between values.
/// Must be implemented to create a Tween.
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

impl Interpolate for Color {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        Color {
            r: lerp(self.r as f32, other.r as f32, t).clamp(0.0, 255.0) as u8,
            g: lerp(self.g as f32, other.g as f32, t).clamp(0.0, 255.0) as u8,
            b: lerp(self.b as f32, other.b as f32, t).clamp(0.0, 255.0) as u8,
            a: lerp(self.a as f32, other.a as f32, t).clamp(0.0, 255.0) as u8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Easing {
    Linear,
    InSine,
    OutSine,
    InOutSine,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InQuart,
    OutQuart,
    InOutQuart,
    InQuint,
    OutQuint,
    InOutQuint,
    InExpo,
    OutExpo,
    InOutExpo,
    InCirc,
    OutCirc,
    InOutCirc,
    InBack,
    OutBack,
    InOutBack,
    InElastic,
    OutElastic,
    InOutElastic,
    InBounce,
    OutBounce,
    InOutBounce,
    Custom(fn(f32) -> f32),
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    pub(crate) fn apply<T: Add<Output = T> + Sub<Output = T> + Mul<f32, Output = T> + Copy>(
        &self,
        t: f32,
        start: T,
        end: T,
    ) -> f32 {
        match self {
            Self::Linear => t,
            Self::InSine => 1.0 - (t * std::f32::consts::PI / 2.0).cos(),
            Self::OutSine => (t * std::f32::consts::PI / 2.0).sin(),
            Self::InOutSine => -((std::f32::consts::PI * t).cos() - 1.0) / 2.0,
            Self::InQuad => t * t,
            Self::OutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            Self::InOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Self::InCubic => t * t * t,
            Self::OutCubic => 1.0 - (1.0 - t).powi(3),
            Self::InOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Self::InQuart => t * t * t * t,
            Self::OutQuart => 1.0 - (1.0 - t).powi(4),
            Self::InOutQuart => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }
            Self::InQuint => t * t * t * t * t,
            Self::OutQuint => 1.0 - (1.0 - t).powi(5),
            Self::InOutQuint => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(5) / 2.0
                }
            }
            Self::InExpo => {
                if t == 0.0 {
                    0.0
                } else {
                    2.0f32.powf(10.0 * t - 10.0)
                }
            }
            Self::OutExpo => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - 2.0f32.powf(-10.0 * t)
                }
            }
            Self::InOutExpo => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    2.0f32.powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - 2.0f32.powf(-20.0 * t + 10.0)) / 2.0
                }
            }
            Self::InCirc => 1.0 - (1.0 - t * t).sqrt(),
            Self::OutCirc => (1.0 - (1.0 - t) * (1.0 - t)).sqrt(),
            Self::InOutCirc => {
                if t < 0.5 {
                    (1.0 - (1.0 - 2.0 * t).powi(2)).sqrt() / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }
            Self::InBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;

                c3 * t * t * t - c1 * t * t
            }
            Self::OutBack => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;

                1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
            }
            Self::InOutBack => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;

                if t < 0.5 {
                    ((2.0 * t).powi(2) * ((c2 + 1.0) * 2.0 * t - c2)) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((c2 + 1.0) * (t * 2.0 - 2.0) + c2) + 2.0) / 2.0
                }
            }
            Self::InElastic => {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;

                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -2.0f32.powf(10.0 * t - 10.0) * (c4 * (t * 10.0 - 10.75)).sin()
                }
            }
            Self::OutElastic => {
                let c4 = (2.0 * std::f32::consts::PI) / 3.0;

                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    2.0f32.powf(-10.0 * t) * (c4 * (t * 10.0 - 0.75)).sin() + 1.0
                }
            }
            Self::InOutElastic => {
                let c5 = (2.0 * std::f32::consts::PI) / 4.5;

                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -(2.0f32.powf(20.0 * t - 10.0) * (c5 * (20.0 * t - 11.125)).sin()) / 2.0
                } else {
                    2.0f32.powf(-20.0 * t + 10.0) * (c5 * (20.0 * t - 11.125)).sin() / 2.0 + 1.0
                }
            }
            Self::InBounce => 1.0 - Self::OutBounce.apply(1.0 - t, start, end),
            Self::OutBounce => {
                let n1 = 7.5625;
                let d1 = 2.75;

                if t < 1.0 / d1 {
                    n1 * t * t
                } else if t < 2.0 / d1 {
                    n1 * (t - 1.5 / d1) * (t - 1.5 / d1) + 0.75
                } else if t < 2.5 / d1 {
                    n1 * (t - 2.25 / d1) * (t - 2.25 / d1) + 0.9375
                } else {
                    n1 * (t - 2.625 / d1) * (t - 2.625 / d1) + 0.984375
                }
            }
            Self::InOutBounce => {
                if t < 0.5 {
                    Self::InBounce.apply(t * 2.0, start, end) / 2.0
                } else {
                    Self::OutBounce.apply(t * 2.0 - 1.0, start, end) / 2.0 + 0.5
                }
            }
            Self::Custom(f) => f(t),
            Self::CubicBezier(x1, y1, x2, y2) => {
                // 1-  https://wikipedia.org/wiki/B%C3%A9zier_curve#Cubic_B%C3%A9zier_curves
                let bezier = |t: f32, p0: f32, p1: f32, p2: f32, p3: f32| -> f32 {
                    let mt = 1.0 - t;
                    mt * mt * mt * p0
                        + 3.0 * mt * mt * t * p1
                        + 3.0 * mt * t * t * p2
                        + t * t * t * p3
                };

                // 1- Derivative of the cubic bezier curve
                let derivative = |t: f32, p0: f32, p1: f32, p2: f32, p3: f32| -> f32 {
                    let mt = 1.0 - t;
                    3.0 * mt * mt * (p1 - p0) + 6.0 * mt * t * (p2 - p1) + 3.0 * t * t * (p3 - p2)
                };

                let mut t_guess = t;
                for _ in 0..10 {
                    let x = bezier(t_guess, 0.0, *x1, *x2, 1.0);
                    let dx = derivative(t_guess, 0.0, *x1, *x2, 1.0);

                    if dx.abs() < 1e-6 {
                        break;
                    }

                    t_guess -= (x - t) / dx;
                }

                bezier(t_guess, 0.0, *y1, *y2, 1.0)
            }
        }
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::Linear
    }
}

impl ToString for Easing {
    fn to_string(&self) -> String {
        match self {
            Easing::Linear => "Linear".to_string(),
            Easing::InSine => "InSine".to_string(),
            Easing::OutSine => "OutSine".to_string(),
            Easing::InOutSine => "InOutSine".to_string(),
            Easing::InQuad => "InQuad".to_string(),
            Easing::OutQuad => "OutQuad".to_string(),
            Easing::InOutQuad => "InOutQuad".to_string(),
            Easing::InCubic => "InCubic".to_string(),
            Easing::OutCubic => "OutCubic".to_string(),
            Easing::InOutCubic => "InOutCubic".to_string(),
            Easing::InQuart => "InQuart".to_string(),
            Easing::OutQuart => "OutQuart".to_string(),
            Easing::InOutQuart => "InOutQuart".to_string(),
            Easing::InQuint => "InQuint".to_string(),
            Easing::OutQuint => "OutQuint".to_string(),
            Easing::InOutQuint => "InOutQuint".to_string(),
            Easing::InExpo => "InExpo".to_string(),
            Easing::OutExpo => "OutExpo".to_string(),
            Easing::InOutExpo => "InOutExpo".to_string(),
            Easing::InCirc => "InCirc".to_string(),
            Easing::OutCirc => "OutCirc".to_string(),
            Easing::InOutCirc => "InOutCirc".to_string(),
            Easing::InBack => "InBack".to_string(),
            Easing::OutBack => "OutBack".to_string(),
            Easing::InOutBack => "InOutBack".to_string(),
            Easing::InElastic => "InElastic".to_string(),
            Easing::OutElastic => "OutElastic".to_string(),
            Easing::InOutElastic => "InOutElastic".to_string(),
            Easing::InBounce => "InBounce".to_string(),
            Easing::OutBounce => "OutBounce".to_string(),
            Easing::InOutBounce => "InOutBounce".to_string(),
            Easing::Custom(_) => "Custom".to_string(),
            Easing::CubicBezier(x1, y1, x2, y2) => {
                format!("CubicBezier({}, {}, {}, {})", x1, y1, x2, y2)
            }
        }
    }
}

pub struct Tween<T: Mul<f32, Output = T> + Add<Output = T> + Sub<Output = T> + Copy> {
    start: T,
    end: T,
    duration: Duration,
    elapsed: f32,
    easing: Easing,

    /// Internals
    running: bool,
    paused: bool,
}

impl<T: Mul<f32, Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Tween<T> {
    pub fn new(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
            easing,
            running: false,
            paused: false,
        }
    }

    pub fn new_and_start(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        let mut tween = Self::new(start, end, duration, easing);
        tween.start();
        tween
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn move_by(&mut self, t: f32) -> T {
        if !self.running || self.paused {
            return self.start;
        }

        self.elapsed += t;

        let dur_f32 = self.duration.as_secs_f32();

        if self.elapsed >= dur_f32 {
            self.running = false;
            self.elapsed = dur_f32;
        }

        let t = self
            .easing
            .apply(self.elapsed / dur_f32, self.start, self.end);

        lerp(self.start, self.end, t)
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration.as_secs_f32()
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn easing(&self) -> Easing {
        self.easing
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.start, &mut self.end);
        self.elapsed = 0.0;
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }
}
