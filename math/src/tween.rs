use crate::Vector;
use macros::{Get, Random, Set, With};
use std::{f32::consts::PI, fmt, time::Duration};

pub trait Lerp: Copy {
    fn lerp(&self, end: &Self, t: f32) -> Self;
}

/// Implements Lerp for types that support Add, Sub, and Mul<f32>
///
/// # Example
/// ```
/// impl_lerp!(f32);
/// impl_lerp!(f64);
/// ```
#[macro_export]
macro_rules! impl_lerp {
    ($type:ty) => {
        impl Lerp for $type {
            fn lerp(&self, end: &Self, t: f32) -> Self {
                let start_f = num::ToPrimitive::to_f32(self).unwrap();
                let end_f = num::ToPrimitive::to_f32(end).unwrap();
                let result = start_f + (end_f - start_f) * t;
                num::FromPrimitive::from_f32(result).unwrap()
            }
        }
    };
}

impl_lerp!(i8);
impl_lerp!(i16);
impl_lerp!(i32);
impl_lerp!(i64);
impl_lerp!(i128);
impl_lerp!(u8);
impl_lerp!(u16);
impl_lerp!(u32);
impl_lerp!(u64);
impl_lerp!(u128);
impl_lerp!(f32);
impl_lerp!(f64);
impl_lerp!(usize);
impl_lerp!(isize);

impl<const N: usize> Lerp for Vector<N> {
    fn lerp(&self, end: &Self, t: f32) -> Self {
        self + (end - self) * t
    }
}

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
pub enum LoopMode {
    #[default]
    None,
    Once,
    Repeat,
    RepeatN(u32),
    Yoyo,
    YoyoN(u32),
}

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
#[derive(Random)]
pub enum Easing {
    #[default]
    Linear,
    QuadIn,
    QuadOut,
    QuadInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuartIn,
    QuartOut,
    QuartInOut,
    QuintIn,
    QuintOut,
    QuintInOut,
    ExpoIn,
    ExpoOut,
    ExpoInOut,
    CircIn,
    CircOut,
    CircInOut,
    BackIn,
    BackOut,
    BackInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl fmt::Display for Easing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,

            Self::QuadIn => t * t,
            Self::QuadOut => t * (2.0 - t),
            Self::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }

            Self::CubicIn => t * t * t,
            Self::CubicOut => {
                let f = t - 1.0;
                f * f * f + 1.0
            }
            Self::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    let f = -2.0 * t + 2.0;
                    1.0 - f * f * f / 2.0
                }
            }

            Self::QuartIn => t * t * t * t,
            Self::QuartOut => {
                let f = t - 1.0;
                1.0 - f * f * f * f
            }
            Self::QuartInOut => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    let f = -2.0 * t + 2.0;
                    1.0 - f * f * f * f / 2.0
                }
            }

            Self::QuintIn => t * t * t * t * t,
            Self::QuintOut => {
                let f = t - 1.0;
                1.0 + f * f * f * f * f
            }
            Self::QuintInOut => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    let f = -2.0 * t + 2.0;
                    1.0 - f * f * f * f * f / 2.0
                }
            }

            Self::ExpoIn => {
                if t == 0.0 {
                    0.0
                } else {
                    (2.0f32).powf(10.0 * t - 10.0)
                }
            }
            Self::ExpoOut => {
                if t == 1.0 {
                    1.0
                } else {
                    1.0 - (2.0f32).powf(-10.0 * t)
                }
            }
            Self::ExpoInOut => {
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    (2.0f32).powf(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - (2.0f32).powf(-20.0 * t + 10.0)) / 2.0
                }
            }

            Self::CircIn => 1.0 - (1.0 - t * t).sqrt(),
            Self::CircOut => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Self::CircInOut => {
                if t < 0.5 {
                    (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0
                } else {
                    ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0
                }
            }

            Self::BackIn => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                c3 * t * t * t - c1 * t * t
            }
            Self::BackOut => {
                let c1 = 1.70158;
                let c3 = c1 + 1.0;
                let f = t - 1.0;
                1.0 + c3 * f * f * f + c1 * f * f
            }
            Self::BackInOut => {
                let c1 = 1.70158;
                let c2 = c1 * 1.525;
                if t < 0.5 {
                    let f = 2.0 * t;
                    (f * f * ((c2 + 1.0) * f - c2)) / 2.0
                } else {
                    let f = 2.0 * t - 2.0;
                    (f * f * ((c2 + 1.0) * f + c2) + 2.0) / 2.0
                }
            }

            Self::ElasticIn => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    -(2.0f32).powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * c4).sin()
                }
            }
            Self::ElasticOut => {
                let c4 = (2.0 * PI) / 3.0;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else {
                    (2.0f32).powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
                }
            }
            Self::ElasticInOut => {
                let c5 = (2.0 * PI) / 4.5;
                if t == 0.0 {
                    0.0
                } else if t == 1.0 {
                    1.0
                } else if t < 0.5 {
                    -((2.0f32).powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0
                } else {
                    ((2.0f32).powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * c5).sin()) / 2.0 + 1.0
                }
            }

            // Bounce (Bouncing ball logic)
            Self::BounceIn => 1.0 - Self::BounceOut.apply(1.0 - t),
            Self::BounceOut => {
                let n1 = 7.5625;
                let n2 = 2.75;

                if t < 1.0 / n2 {
                    n1 * t * t
                } else if t < 2.0 / n2 {
                    let t = t - 1.5 / n2;
                    n1 * t * t + 0.75
                } else if t < 2.5 / n2 {
                    let t = t - 2.25 / n2;
                    n1 * t * t + 0.9375
                } else {
                    let t = t - 2.625 / n2;
                    n1 * t * t + 0.984375
                }
            }
            Self::BounceInOut => {
                if t < 0.5 {
                    (1.0 - Self::BounceOut.apply(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + Self::BounceOut.apply(2.0 * t - 1.0)) / 2.0
                }
            }
        }
    }
}

#[derive(Get, Set, With)]
pub struct Tween<T: Lerp> {
    #[get]
    #[with(name = "with_start")]
    a: T,

    #[get(name = "target")]
    #[set(name = "set_target")]
    #[with(name = "with_target")]
    b: T,

    #[get(copied)]
    #[set]
    #[with]
    easing: Easing,

    #[get(copied)]
    #[set]
    #[with]
    duration: f32,

    #[get(copied)]
    elapsed: f32,

    #[get(copied)]
    paused: bool,

    #[get(copied)]
    #[with]
    loop_mode: LoopMode,
    loop_counter: u32,
    yoyo_forward: bool,

    // Callbacks
    on_start: Option<Box<dyn FnMut(&mut Self) + Send>>,
    on_complete: Option<Box<dyn FnMut(&mut Self) + Send>>,
}

impl<T: Lerp> Tween<T> {
    pub fn new(start: T, end: T, easing: Easing, duration: Duration) -> Self {
        Tween {
            a: start,
            b: end,
            easing,
            duration: duration.as_secs_f32(),
            elapsed: 0.0,
            paused: true,
            loop_mode: LoopMode::default(),
            loop_counter: 0,
            yoyo_forward: true,
            on_start: None,
            on_complete: None,
        }
    }

    /// Set the callback to be called when the tween starts
    pub fn on_start<F: FnMut(&mut Self) + Send + 'static>(&mut self, callback: F) {
        self.on_start = Some(Box::new(callback));
    }

    /// Set the callback to be called when the tween completes
    pub fn on_complete<F: FnMut(&mut Self) + Send + 'static>(&mut self, callback: F) {
        self.on_complete = Some(Box::new(callback));
    }

    /// Sample at a specific normalized time (0.0 to 1.0)
    #[inline]
    pub fn sample(&self, t: f32) -> T {
        let t = self.easing.apply(t.clamp(0.0, 1.0));
        self.a.lerp(&self.b, t)
    }

    /// Update with delta time, returns current value
    #[inline]
    pub fn update(&mut self, dt: f32) {
        let was_complete = self.is_complete();

        if !self.paused {
            self.elapsed = (self.elapsed + dt).min(self.duration);
        }

        if !was_complete && self.is_complete() {
            if let Some(mut callback) = self.on_complete.take() {
                callback(self);

                self.on_complete = Some(callback);
            }

            match self.loop_mode {
                LoopMode::Once => self.elapsed = self.duration,
                LoopMode::Repeat => self.elapsed %= self.duration,
                LoopMode::RepeatN(n) if self.loop_counter < n => {
                    self.elapsed %= self.duration;
                    self.loop_counter += 1;
                }

                LoopMode::Yoyo => {
                    self.reverse();
                    self.elapsed = 0.0;
                    self.yoyo_forward = !self.yoyo_forward;
                }

                LoopMode::YoyoN(n) => {
                    // Each complete yoyo = forward + backward (2 reversals)
                    let complete_yoyos = self.loop_counter / 2;
                    if complete_yoyos < n {
                        self.reverse();
                        self.elapsed = 0.0;
                        self.yoyo_forward = !self.yoyo_forward;
                        self.loop_counter += 1;
                    } else {
                        self.elapsed = self.duration;
                    }
                }

                _ => self.elapsed = self.duration,
            }
        }
    }

    #[inline]
    pub fn value(&self) -> T {
        self.sample(self.elapsed / self.duration)
    }

    /// Start/resume the tween
    #[inline]
    pub fn start(&mut self) {
        self.paused = false;

        if let Some(mut callback) = self.on_start.take() {
            callback(self);

            self.on_start = Some(callback);
        }
    }

    /// Pause the tween
    #[inline]
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Toggle pause state
    #[inline]
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Reverse the tween by swapping start and end
    #[inline]
    pub fn reverse(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
        self.elapsed = self.duration - self.elapsed;
    }

    /// Play tween in reverse direction from current position
    #[inline]
    pub fn play_reverse(&mut self) {
        self.reverse();
        self.reset();
        self.start();
    }

    /// Toggle between forward and reverse playback
    #[inline]
    pub fn toggle_direction(&mut self) {
        if self.is_complete() || self.is_paused() {
            self.play_reverse();
        }
    }

    /// Check if tween is complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Check if tween is paused
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Reset the tween to start
    #[inline]
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.loop_counter = 0;
        self.yoyo_forward = true;
    }

    /// Get normalized progress (0.0 to 1.0)
    #[inline]
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).min(1.0)
    }
}
