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

/// A list of easing functions that can be used to interpolate values.
/// Basically a function that takes a value between 0.0 and 1.0 and returns
/// a value following the easing function's curve.
///
/// # References
/// - [Easing functions](https://easings.net/)
#[derive(Clone, Copy)]
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
    pub fn apply(&self, t: f32) -> f32 {
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
            Self::InBounce => 1.0 - Self::OutBounce.apply(1.0 - t),
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
                    Self::InBounce.apply(t * 2.0) / 2.0
                } else {
                    Self::OutBounce.apply(t * 2.0 - 1.0) / 2.0 + 0.5
                }
            }
            Self::Custom(f) => f(t),

            Self::CubicBezier(p0, p1, p2, p3) => {
                p0 * (1.0 - t).powi(3)
                    + 3.0 * p1 * t * (1.0 - t).powi(2)
                    + 3.0 * p2 * t.powi(2) * (1.0 - t)
                    + p3 * t.powi(3)
            }
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Linear => "Linear".to_string(),
            Self::InSine => "InSine".to_string(),
            Self::OutSine => "OutSine".to_string(),
            Self::InOutSine => "InOutSine".to_string(),
            Self::InQuad => "InQuad".to_string(),
            Self::OutQuad => "OutQuad".to_string(),
            Self::InOutQuad => "InOutQuad".to_string(),
            Self::InCubic => "InCubic".to_string(),
            Self::OutCubic => "OutCubic".to_string(),
            Self::InOutCubic => "InOutCubic".to_string(),
            Self::InQuart => "InQuart".to_string(),
            Self::OutQuart => "OutQuart".to_string(),
            Self::InOutQuart => "InOutQuart".to_string(),
            Self::InQuint => "InQuint".to_string(),
            Self::OutQuint => "OutQuint".to_string(),
            Self::InOutQuint => "InOutQuint".to_string(),
            Self::InExpo => "InExpo".to_string(),
            Self::OutExpo => "OutExpo".to_string(),
            Self::InOutExpo => "InOutExpo".to_string(),
            Self::InCirc => "InCirc".to_string(),
            Self::OutCirc => "OutCirc".to_string(),
            Self::InOutCirc => "InOutCirc".to_string(),
            Self::InBack => "InBack".to_string(),
            Self::OutBack => "OutBack".to_string(),
            Self::InOutBack => "InOutBack".to_string(),
            Self::InElastic => "InElastic".to_string(),
            Self::OutElastic => "OutElastic".to_string(),
            Self::InOutElastic => "InOutElastic".to_string(),
            Self::InBounce => "InBounce".to_string(),
            Self::OutBounce => "OutBounce".to_string(),
            Self::InOutBounce => "InOutBounce".to_string(),
            Self::Custom(_) => "Custom".to_string(),
            Self::CubicBezier(p0, p1, p2, p3) => {
                format!("CubicBezier({}, {}, {}, {})", p0, p1, p2, p3)
            }
        }
    }

    pub fn all() -> Vec<Easing> {
        vec![
            Self::Linear,
            Self::InSine,
            Self::OutSine,
            Self::InOutSine,
            Self::InQuad,
            Self::OutQuad,
            Self::InOutQuad,
            Self::InCubic,
            Self::OutCubic,
            Self::InOutCubic,
            Self::InQuart,
            Self::OutQuart,
            Self::InOutQuart,
            Self::InQuint,
            Self::OutQuint,
            Self::InOutQuint,
            Self::InExpo,
            Self::OutExpo,
            Self::InOutExpo,
            Self::InCirc,
            Self::OutCirc,
            Self::InOutCirc,
            Self::InBack,
            Self::OutBack,
            Self::InOutBack,
            Self::InElastic,
            Self::OutElastic,
            Self::InOutElastic,
            Self::InBounce,
            Self::OutBounce,
            Self::InOutBounce,
        ]
    }
}

/// A structs that takes in a value and interpolates it
/// to another value over a certain duration.
/// This means that you can change a value from one to another
/// while respecting a certain easing function.
///
/// This is very useful for animations and transitions,
/// like the opening of a menu, or the rotation of a sprite.
pub struct Tween<T: Interpolate + Copy> {
    /// The value at the start of the tween
    start: T,
    /// The target value, meaning the start value will interpolate to this value
    end: T,
    /// The duration of the tween
    duration: Duration,
    /// The time elapsed since the tween started
    elapsed: f32,
    /// The easing function to use
    easing: Easing,
    /// Internal flag to know if the tween is running or not
    running: bool,

    /// A value needed to reset / reverse the tween
    original_start: T,
}

impl<T: Interpolate + Copy> Tween<T> {
    /// Creates a new tween with the given start and end values.
    /// It will not start automatically, you need to call `start` to start it.
    pub fn new(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        Self {
            start,
            end,
            duration,
            running: false,
            elapsed: 0.0,
            easing,
            original_start: start,
        }
    }

    /// Creates a new tween with the given start and end values.
    /// It will start automatically.
    pub fn new_and_start(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        Self {
            start,
            end,
            duration,
            running: true,
            elapsed: 0.0,
            easing,
            original_start: start,
        }
    }

    /// Returns true if the tween is running, false otherwise.
    pub fn paused(&self) -> bool {
        !self.running
    }

    /// Pauses the tween.
    pub fn pause(&mut self) {
        self.running = false;
    }

    /// Starts / resumes the tween.
    pub fn start(&mut self) {
        self.running = true;
    }

    /// A function that returns the value of the tween at the current time.
    /// This function should be called every frame to update the tween.
    pub fn update(&mut self, dt: f32) -> T {
        if self.running {
            self.elapsed += dt;
        }

        let dur = self.duration.as_secs_f32();

        if self.elapsed > dur {
            self.elapsed = dur;
        }

        self.start
            .interpolate(&self.end, self.easing.apply(self.elapsed / dur))
    }

    /// Returns the duration of the tween.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the time elapsed since the tween started.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Returns the easing function of the tween.
    pub fn easing(&self) -> Easing {
        self.easing
    }

    /// Returns the end value of the tween.
    pub fn target(&self) -> T {
        self.end
    }

    /// Sets the end value of the tween.
    /// Useful for changing the target value while running.
    pub fn set_target(&mut self, target: T) {
        self.end = target;
    }

    /// Resets the tween to its original state.
    /// Does not start the tween.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.start = self.original_start;
        self.pause();
    }

    /// Resets the tween to its original state and starts it.
    pub fn restart(&mut self) {
        self.elapsed = 0.0;
        self.start = self.original_start;
        self.start();
    }

    /// Reverses the tween.
    /// Does not start the tween.
    pub fn reverse(&mut self) {
        let temp = self.start;
        self.start = self.end;
        self.end = temp;
        self.elapsed = 0.0;
        self.pause();
    }

    /// Reverse the tween and starts it.
    pub fn reverse_and_start(&mut self) {
        let temp = self.start;
        self.start = self.end;
        self.end = temp;
        self.elapsed = 0.0;
        self.start();
    }

    /// Returns true if the tween is finished, false otherwise.
    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration.as_secs_f32()
    }
}
