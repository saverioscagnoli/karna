use std::f32::consts::PI;
use std::fmt;
use std::time::Duration;

use crate::Vector;

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
                let start_f = num_traits::ToPrimitive::to_f32(self).unwrap();
                let end_f = num_traits::ToPrimitive::to_f32(end).unwrap();
                let result = start_f + (end_f - start_f) * t;

                num_traits::FromPrimitive::from_f32(result).unwrap_or_else(|| {
                    if result < start_f.min(end_f) {
                        (*self).min(*end)
                    } else {
                        (*self).max(*end)
                    }
                })
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

impl<const N: usize> Lerp for Vector<N, f32> {
    fn lerp(&self, end: &Self, t: f32) -> Self {
        self + (end - self) * t
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum LoopMode {
    #[default]
    None,
    Once,
    /// Loops forever, carrying frame overshoot so the period is exact.
    Repeat,
    /// Plays n + 1 times total (the first play, then n repeats).
    RepeatN(u32),
    Yoyo,
    /// n complete there-and-back cycles.
    YoyoN(u32),
}

#[derive(Clone, Copy)]
#[derive(Default)]
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
    Custom(fn(f32) -> f32),
}

impl fmt::Debug for Easing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(_) => write!(f, "Custom"),
            _ => write!(f, "{}", self),
        }
    }
}

impl fmt::Display for Easing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Linear => write!(f, "Linear"),
            Self::QuadIn => write!(f, "QuadIn"),
            Self::QuadOut => write!(f, "QuadOut"),
            Self::QuadInOut => write!(f, "QuadInOut"),
            Self::CubicIn => write!(f, "CubicIn"),
            Self::CubicOut => write!(f, "CubicOut"),
            Self::CubicInOut => write!(f, "CubicInOut"),
            Self::QuartIn => write!(f, "QuartIn"),
            Self::QuartOut => write!(f, "QuartOut"),
            Self::QuartInOut => write!(f, "QuartInOut"),
            Self::QuintIn => write!(f, "QuintIn"),
            Self::QuintOut => write!(f, "QuintOut"),
            Self::QuintInOut => write!(f, "QuintInOut"),
            Self::ExpoIn => write!(f, "ExpoIn"),
            Self::ExpoOut => write!(f, "ExpoOut"),
            Self::ExpoInOut => write!(f, "ExpoInOut"),
            Self::CircIn => write!(f, "CircIn"),
            Self::CircOut => write!(f, "CircOut"),
            Self::CircInOut => write!(f, "CircInOut"),
            Self::BackIn => write!(f, "BackIn"),
            Self::BackOut => write!(f, "BackOut"),
            Self::BackInOut => write!(f, "BackInOut"),
            Self::ElasticIn => write!(f, "ElasticIn"),
            Self::ElasticOut => write!(f, "ElasticOut"),
            Self::ElasticInOut => write!(f, "ElasticInOut"),
            Self::BounceIn => write!(f, "BounceIn"),
            Self::BounceOut => write!(f, "BounceOut"),
            Self::BounceInOut => write!(f, "BounceInOut"),
            Self::Custom(_) => write!(f, "Custom"),
        }
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

            Self::Custom(f) => f(t),
        }
    }
}

pub struct Tween<T: Lerp> {
    a: T,
    b: T,
    easing: Easing,
    duration: f32,
    elapsed: f32,
    paused: bool,
    loop_mode: LoopMode,
    loop_counter: u32,
    forward: bool,

    // Callbacks
    on_start: Option<Box<dyn FnMut(&mut Self) + Send>>,
    on_complete: Option<Box<dyn FnMut(&mut Self) + Send>>,
}

impl<T: Lerp + std::fmt::Debug> std::fmt::Debug for Tween<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tween")
            .field("a", &self.a)
            .field("b", &self.b)
            .field("easing", &self.easing)
            .field("duration", &self.duration)
            .field("elapsed", &self.elapsed)
            .field("paused", &self.paused)
            .field("loop_mode", &self.loop_mode)
            .field("loop_counter", &self.loop_counter)
            .field("forward", &self.forward)
            .finish()
    }
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
            forward: true,
            on_start: None,
            on_complete: None,
        }
    }

    pub fn from(&self) -> &T {
        &self.a
    }

    pub fn target(&self) -> &T {
        &self.b
    }

    pub fn set_target(&mut self, t: T) {
        self.b = t;
    }

    pub fn with_target(mut self, t: T) -> Self {
        self.b = t;
        self
    }

    pub fn easing(&self) -> Easing {
        self.easing
    }

    pub fn set_easing(&mut self, e: Easing) {
        self.easing = e;
    }

    pub fn with_easing(mut self, e: Easing) -> Self {
        self.easing = e;
        self
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn loop_mode(&self) -> LoopMode {
        self.loop_mode
    }

    pub fn set_loop_mode(&mut self, mode: LoopMode) {
        self.loop_mode = mode;
    }

    pub fn with_loop_mode(mut self, mode: LoopMode) -> Self {
        self.loop_mode = mode;
        self
    }

    /// Set the callback to be called when the tween starts
    pub fn on_start<F: FnMut(&mut Self) + Send + 'static>(&mut self, callback: F) {
        self.on_start = Some(Box::new(callback));
    }

    /// Set the callback to be called when the tween completes
    pub fn on_complete<F: FnMut(&mut Self) + Send + 'static>(&mut self, callback: F) {
        self.on_complete = Some(Box::new(callback));
    }

    /// Sample at a specific normalized time (0.0 to 1.0), honoring the
    /// current playback direction.
    #[inline]
    pub fn sample(&self, t: f32) -> T {
        let t = self.easing.apply(t.clamp(0.0, 1.0));
        if self.forward {
            self.a.lerp(&self.b, t)
        } else {
            self.b.lerp(&self.a, t)
        }
    }

    /// Advance by delta time and return the current value.
    ///
    /// `on_complete` fires at most once per call, even if `dt` spans
    /// multiple loop iterations.
    #[inline]
    pub fn update(&mut self, dt: f32) -> T {
        // FIX #2: zero/negative duration is an instantly-complete tween;
        // never divide by it.
        if self.paused || self.duration <= 0.0 || self.is_complete() {
            return self.value();
        }

        self.elapsed += dt;

        if self.elapsed >= self.duration {
            if let Some(mut callback) = self.on_complete.take() {
                callback(self);

                self.on_complete = Some(callback);
            }

            let over = (self.elapsed - self.duration) % self.duration;

            match self.loop_mode {
                LoopMode::Once => self.elapsed = self.duration,
                LoopMode::Repeat => self.elapsed = over,
                LoopMode::RepeatN(n) if self.loop_counter < n => {
                    self.elapsed = over;
                    self.loop_counter += 1;
                }

                LoopMode::Yoyo => {
                    self.forward = !self.forward;
                    self.elapsed = over;
                }

                LoopMode::YoyoN(n) => {
                    // Each complete yoyo = forward + backward (2 reversals)
                    let complete_yoyos = self.loop_counter / 2;
                    if complete_yoyos < n {
                        self.forward = !self.forward;
                        self.elapsed = over;
                        self.loop_counter += 1;
                    } else {
                        self.elapsed = self.duration;
                    }
                }

                _ => self.elapsed = self.duration,
            }
        }

        self.value()
    }

    #[inline]
    pub fn animate(&mut self, target: &mut T, dt: f32) {
        *target = self.update(dt);
    }

    #[inline]
    pub fn value(&self) -> T {
        self.sample(self.progress())
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

    /// Reverse playback direction from the current position. The value
    /// is continuous across the reversal (smooth turnaround), and the
    /// endpoints `a`/`b` are left untouched.
    #[inline]
    pub fn reverse(&mut self) {
        self.forward = !self.forward;
        self.elapsed = (self.duration - self.elapsed).max(0.0);
    }

    /// Play tween in reverse direction from the beginning
    #[inline]
    pub fn play_reverse(&mut self) {
        let forward = self.forward;
        self.reset();
        self.forward = !forward;
        self.start();
    }

    /// FIX #5: reverse direction and keep playing — works mid-flight
    /// (smooth turnaround) as well as when paused or complete.
    #[inline]
    pub fn toggle_direction(&mut self) {
        self.reverse();
        self.paused = false;
    }

    /// Check if tween is complete
    #[inline]
    pub fn is_complete(&self) -> bool {
        match self.loop_mode {
            // Infinite loops never complete.
            LoopMode::Repeat | LoopMode::Yoyo => false,
            _ => self.duration <= 0.0 || self.elapsed >= self.duration,
        }
    }

    /// Check if tween is paused
    #[inline]
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Reset the tween to start (forward direction, loop count cleared)
    #[inline]
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.loop_counter = 0;
        self.forward = true;
    }

    /// Get normalized progress (0.0 to 1.0)
    #[inline]
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).min(1.0)
        }
    }
}
