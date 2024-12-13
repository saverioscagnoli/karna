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

pub struct Tween<T: Interpolate + Clone> {
    start: T,
    end: T,
    duration: Duration,
    elapsed: f32,
    easing: Easing,

    original_start: T,
}

impl<T: Interpolate + Clone> Tween<T> {
    pub fn new(start: T, end: T, duration: Duration, easing: Easing) -> Self {
        let original_start = start.clone();

        Self {
            start,
            end,
            duration,
            elapsed: 0.0,
            easing,
            original_start,
        }
    }

    pub fn update(&mut self, dt: f32) -> T {
        self.elapsed += dt;

        let dur = self.duration.as_secs_f32();

        if self.elapsed > dur {
            self.elapsed = dur;
        }

        self.start
            .interpolate(&self.end, self.easing.apply(self.elapsed / dur))
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn easing(&self) -> Easing {
        self.easing
    }

    pub fn set_target(&mut self, target: T) {
        self.end = target;
    }

    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.start = self.original_start.clone();
    }

    pub fn reverse(&mut self) {
        let temp = self.start.clone();
        self.start = self.end.clone();
        self.end = temp;
        self.elapsed = 0.0;
    }

    pub fn finished(&self) -> bool {
        self.elapsed >= self.duration.as_secs_f32()
    }
}
