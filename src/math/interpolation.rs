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

/// A tween is a smooth transition between two values over a period of time.
/// It uses an easing function to determine how the transition should look like.
/// For more information on easing functions, check the `Easing` enum.
///
/// The `T` type must implement scalar add, sub and mul operations.
/// Basically it must be a type that can be added, subtracted and multiplied by a float.
/// Works with numbers, vectors, colors, etc.
///
/// # Example
/// ```no_run
/// let start = Vec2::new(0.0, 0.0);
/// let end = Vec2::new(100.0, 100.0);
///
/// let mut tween = Tween::new(start, end, Duration::from_secs(2), Easing::InOutQuad);
///
/// loop {
///    let pos = tween.move_by(delta);
///
///     if tween.is_finished() {
///        break;
///    }
/// }
/// ```
///
/// This will move the vector from `(0, 0)` to `(100, 100)` over a period of 2 seconds using the `InOutQuad` easing function.
///
pub struct Tween<T: Mul<f32, Output = T> + Add<Output = T> + Sub<Output = T> + Copy> {
    /// The start value of the tween
    start: T,
    /// The current value of the tween
    curr: T,
    /// The end value of the tween
    end: T,
    /// The duration of the tween
    duration: Duration,
    /// The time elapsed since the tween started
    elapsed: f32,
    /// The easing function to use
    easing: Easing,

    /// Internals
    running: bool,
    paused: bool,
}

impl<T: Mul<f32, Output = T> + Add<Output = T> + Sub<Output = T> + Copy> Tween<T> {
    /// Creates a new tween with the given start and end values, duration and easing function.
    pub fn new(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        Self {
            start,
            curr: start,
            end,
            duration,
            elapsed: 0.0,
            easing,
            running: false,
            paused: false,
        }
    }

    /// Creates a new tween with the given start and end values, duration and easing function and starts it immediately.
    pub fn new_and_start(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        let mut tween = Self::new(start, end, duration, easing);
        tween.start();
        tween
    }

    /// Starts the tween
    pub fn start(&mut self) {
        self.running = true;
    }

    /// Pauses the tween
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the tween
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Updates the tween by the given time delta.
    pub fn move_by(&mut self, t: f32) -> T {
        if !self.running || self.paused {
            return self.curr;
        }

        self.elapsed += t;

        let dur_f32 = self.duration.as_secs_f32();

        if self.elapsed >= dur_f32 {
            self.running = false;
            self.elapsed = dur_f32;
        }

        let t = self
            .easing
            .apply(self.elapsed / dur_f32, self.curr, self.end);

        lerp(self.curr, self.end, t)
    }

    /// Returns the start value of the tween
    pub fn start_value(&self) -> T {
        self.start
    }

    /// Returns the end value of the tween
    pub fn target(&self) -> T {
        self.end
    }

    /// Returns the value of the tween at the given time.
    pub fn value(&self) -> T {
        self.curr
    }

    /// Checks if the tween has finished
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration.as_secs_f32()
    }

    /// Checks if the tween is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Checks if the tween is paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Returns the easing function of the tween
    pub fn easing(&self) -> Easing {
        self.easing
    }

    /// Returns how much time has elapsed since the tween started
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Returns the duration of the tween
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Resets the tween and stops it
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.running = false;
        self.paused = false;
    }

    /// Reverses the tween and keeps it running
    pub fn reset_and_start(&mut self) {
        self.reset();
        self.start();
    }

    /// Reverses the tween and stops it
    pub fn reverse(&mut self) {
        let temp = self.curr;
        self.curr = self.end;
        self.end = temp;

        self.elapsed = 0.0;
        self.running = false;
        self.paused = false;
    }

    /// Reverses the tween and keeps it running
    pub fn reverse_and_start(&mut self) {
        self.reverse();
        self.start();
    }
}
