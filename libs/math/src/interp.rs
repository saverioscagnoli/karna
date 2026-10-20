use core::f32;
use core::time::Duration;

use crate::SdlFloat;
use crate::Size;
use crate::Vector;

pub trait Lerp: Copy {
    fn lerp(&self, to: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        self + (to - self) * t
    }
}

impl Lerp for f64 {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        self + (to - self) * t as f64
    }
}

impl<const N: usize> Lerp for Vector<N, f32> {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        let mut out = Self::zero();

        for i in 0..N {
            out[i] = self[i].lerp(&to[i], t);
        }

        out
    }
}

impl Lerp for Size<f32> {
    fn lerp(&self, to: &Self, t: f32) -> Self {
        let mut out = Size::zero();

        out.width = self.width.lerp(&to.width, t);
        out.height = self.height.lerp(&to.height, t);
        out
    }
}

fn pow2(x: f32) -> f32 {
    (x * f32::consts::LN_2).sdl_exp()
}

fn sqrt(x: f32) -> f32 {
    x.sdl_max(0.0).sdl_sqrt()
}

fn bounce_out(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;

    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

fn bezier_coeffs(p1: f32, p2: f32) -> (f32, f32, f32) {
    let c = 3.0 * p1;
    let b = 3.0 * (p2 - p1) - c;
    let a = 1.0 - c - b;
    (a, b, c)
}

fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    if x <= 0.0 {
        return 0.0;
    }

    if x >= 1.0 {
        return 1.0;
    }

    let (ax, bx, cx) = bezier_coeffs(x1.sdl_clamp(0.0, 1.0), x2.sdl_clamp(0.0, 1.0));
    let (ay, by, cy) = bezier_coeffs(y1, y2);

    let sample_x = |s: f32| ((ax * s + bx) * s + cx) * s;
    let slope_x = |s: f32| (3.0 * ax * s + 2.0 * bx) * s + cx;
    let sample_y = |s: f32| ((ay * s + by) * s + cy) * s;

    let mut s = x;

    for _ in 0..8 {
        let err = sample_x(s) - x;

        if err.sdl_abs() < 1e-6 {
            return sample_y(s);
        }

        let d = slope_x(s);

        if d.sdl_abs() < 1e-6 {
            break;
        }

        s -= err / d;
    }

    let (mut lo, mut hi) = (0.0, 1.0);
    s = x;

    for _ in 0..32 {
        let v = sample_x(s);

        if (v - x).sdl_abs() < 1e-6 {
            break;
        }

        if v < x {
            lo = s;
        } else {
            hi = s;
        }

        s = (lo + hi) / 2.0;
    }

    sample_y(s)
}

const BACK_C1: f32 = 1.70158;
const BACK_C2: f32 = BACK_C1 * 1.525;
const BACK_C3: f32 = BACK_C1 + 1.0;
const ELASTIC_C4: f32 = f32::consts::TAU / 3.0;
const ELASTIC_C5: f32 = f32::consts::TAU / 4.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ease {
    Linear,
    SineIn,
    SineOut,
    SineInOut,
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
    CubicBezier(f32, f32, f32, f32),
}

impl Ease {
    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Ease::CubicBezier(x1.sdl_clamp(0.0, 1.0), y1, x2.sdl_clamp(0.0, 1.0), y2)
    }

    pub const ALL: [Ease; 31] = [
        Ease::Linear,
        Ease::SineIn,
        Ease::SineOut,
        Ease::SineInOut,
        Ease::QuadIn,
        Ease::QuadOut,
        Ease::QuadInOut,
        Ease::CubicIn,
        Ease::CubicOut,
        Ease::CubicInOut,
        Ease::QuartIn,
        Ease::QuartOut,
        Ease::QuartInOut,
        Ease::QuintIn,
        Ease::QuintOut,
        Ease::QuintInOut,
        Ease::ExpoIn,
        Ease::ExpoOut,
        Ease::ExpoInOut,
        Ease::CircIn,
        Ease::CircOut,
        Ease::CircInOut,
        Ease::BackIn,
        Ease::BackOut,
        Ease::BackInOut,
        Ease::ElasticIn,
        Ease::ElasticOut,
        Ease::ElasticInOut,
        Ease::BounceIn,
        Ease::BounceOut,
        Ease::BounceInOut,
    ];

    pub fn apply(self, t: f32) -> f32 {
        let pi = f32::consts::PI;
        let u = 1.0 - t;
        let v = -2.0 * t + 2.0;

        match self {
            Ease::Linear => t,

            Ease::SineIn => 1.0 - (t * pi / 2.0).sdl_cos(),
            Ease::SineOut => (t * pi / 2.0).sdl_sin(),
            Ease::SineInOut => -((pi * t).sdl_cos() - 1.0) / 2.0,

            Ease::QuadIn => t * t,
            Ease::QuadOut => 1.0 - u * u,
            Ease::QuadInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - v * v / 2.0
                }
            }

            Ease::CubicIn => t * t * t,
            Ease::CubicOut => 1.0 - u * u * u,
            Ease::CubicInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - v * v * v / 2.0
                }
            }

            Ease::QuartIn => t * t * t * t,
            Ease::QuartOut => 1.0 - u * u * u * u,
            Ease::QuartInOut => {
                if t < 0.5 {
                    8.0 * t * t * t * t
                } else {
                    1.0 - v * v * v * v / 2.0
                }
            }

            Ease::QuintIn => t * t * t * t * t,
            Ease::QuintOut => 1.0 - u * u * u * u * u,
            Ease::QuintInOut => {
                if t < 0.5 {
                    16.0 * t * t * t * t * t
                } else {
                    1.0 - v * v * v * v * v / 2.0
                }
            }

            Ease::ExpoIn => {
                if t <= 0.0 {
                    0.0
                } else {
                    pow2(10.0 * t - 10.0)
                }
            }
            Ease::ExpoOut => {
                if t >= 1.0 {
                    1.0
                } else {
                    1.0 - pow2(-10.0 * t)
                }
            }
            Ease::ExpoInOut => {
                if t <= 0.0 {
                    0.0
                } else if t >= 1.0 {
                    1.0
                } else if t < 0.5 {
                    pow2(20.0 * t - 10.0) / 2.0
                } else {
                    (2.0 - pow2(-20.0 * t + 10.0)) / 2.0
                }
            }

            Ease::CircIn => 1.0 - sqrt(1.0 - t * t),
            Ease::CircOut => sqrt(1.0 - u * u),
            Ease::CircInOut => {
                if t < 0.5 {
                    (1.0 - sqrt(1.0 - 4.0 * t * t)) / 2.0
                } else {
                    (sqrt(1.0 - v * v) + 1.0) / 2.0
                }
            }

            Ease::BackIn => BACK_C3 * t * t * t - BACK_C1 * t * t,
            Ease::BackOut => {
                let w = t - 1.0;
                1.0 + BACK_C3 * w * w * w + BACK_C1 * w * w
            }
            Ease::BackInOut => {
                if t < 0.5 {
                    let w = 2.0 * t;
                    w * w * ((BACK_C2 + 1.0) * w - BACK_C2) / 2.0
                } else {
                    let w = 2.0 * t - 2.0;
                    (w * w * ((BACK_C2 + 1.0) * w + BACK_C2) + 2.0) / 2.0
                }
            }

            Ease::ElasticIn => {
                if t <= 0.0 {
                    0.0
                } else if t >= 1.0 {
                    1.0
                } else {
                    -pow2(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * ELASTIC_C4).sdl_sin()
                }
            }
            Ease::ElasticOut => {
                if t <= 0.0 {
                    0.0
                } else if t >= 1.0 {
                    1.0
                } else {
                    pow2(-10.0 * t) * ((t * 10.0 - 0.75) * ELASTIC_C4).sdl_sin() + 1.0
                }
            }
            Ease::ElasticInOut => {
                if t <= 0.0 {
                    0.0
                } else if t >= 1.0 {
                    1.0
                } else if t < 0.5 {
                    -(pow2(20.0 * t - 10.0) * ((20.0 * t - 11.125) * ELASTIC_C5).sdl_sin()) / 2.0
                } else {
                    pow2(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * ELASTIC_C5).sdl_sin() / 2.0
                        + 1.0
                }
            }

            Ease::BounceIn => 1.0 - bounce_out(1.0 - t),
            Ease::BounceOut => bounce_out(t),
            Ease::BounceInOut => {
                if t < 0.5 {
                    (1.0 - bounce_out(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + bounce_out(2.0 * t - 1.0)) / 2.0
                }
            }

            Ease::CubicBezier(x1, y1, x2, y2) => cubic_bezier(x1, y1, x2, y2, t),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Repeat {
    Once,
    Loop,
    PingPong,
}

#[derive(Debug, Clone, Copy)]
pub struct Tween<T: Lerp> {
    from: T,
    to: T,
    duration: f32,
    elapsed: f32,
    ease: Ease,
    repeat: Repeat,
}

impl<T: Lerp> Tween<T> {
    pub fn new(from: T, to: T, duration: Duration) -> Self {
        Self {
            from,
            to,
            duration: duration.as_secs_f32(),
            elapsed: 0.0,
            ease: Ease::Linear,
            repeat: Repeat::Once,
        }
    }

    pub fn with_from(mut self, from: T) -> Self {
        self.from = from;
        self
    }

    pub fn with_to(mut self, to: T) -> Self {
        self.to = to;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration.as_secs_f32();
        self
    }

    pub fn with_ease(mut self, ease: Ease) -> Self {
        self.ease = ease;
        self
    }

    pub fn with_repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    #[inline]
    pub fn set_from(&mut self, from: T) {
        self.from = from;
    }

    #[inline]
    pub fn set_to(&mut self, to: T) {
        self.to = to;
    }

    #[inline]
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration.as_secs_f32();
    }

    #[inline]
    pub fn set_ease(&mut self, ease: Ease) {
        self.ease = ease;
    }

    #[inline]
    pub fn set_repeat(&mut self, repeat: Repeat) {
        self.repeat = repeat;
    }

    fn phase(&self) -> f32 {
        if self.duration <= 0.0 {
            return 1.0;
        }

        let t = self.elapsed / self.duration;

        match self.repeat {
            Repeat::Once => t.min(1.0),
            Repeat::Loop => t % 1.0,
            Repeat::PingPong => {
                let c = t % 2.0;
                if c > 1.0 { 2.0 - c } else { c }
            }
        }
    }

    pub fn value(&self) -> T {
        if self.is_finished() {
            return self.to;
        }

        self.from.lerp(&self.to, self.ease.apply(self.phase()))
    }

    pub fn update(&mut self, dt: f32) -> T {
        self.elapsed += dt;

        match self.repeat {
            Repeat::Once => self.elapsed = self.elapsed.min(self.duration),
            Repeat::Loop if self.duration > 0.0 => self.elapsed %= self.duration,
            Repeat::PingPong if self.duration > 0.0 => self.elapsed %= self.duration * 2.0,
            _ => {}
        }

        self.value()
    }

    #[inline]
    pub fn progress(&self) -> f32 {
        self.phase()
    }

    #[inline]
    pub fn is_finished(&self) -> bool {
        matches!(self.repeat, Repeat::Once) && self.elapsed >= self.duration
    }

    #[inline]
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    #[inline]
    pub fn retarget(&mut self, to: T) {
        self.from = self.value();
        self.to = to;
        self.elapsed = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Vector2;

    const EPS: f32 = 1e-4;

    fn approx(a: f32, b: f32) -> bool {
        let d = if a > b { a - b } else { b - a };
        d < EPS
    }

    fn secs(s: f32) -> Duration {
        Duration::from_secs_f32(s)
    }

    const PAIRS: [(Ease, Ease); 10] = [
        (Ease::SineIn, Ease::SineOut),
        (Ease::QuadIn, Ease::QuadOut),
        (Ease::CubicIn, Ease::CubicOut),
        (Ease::QuartIn, Ease::QuartOut),
        (Ease::QuintIn, Ease::QuintOut),
        (Ease::ExpoIn, Ease::ExpoOut),
        (Ease::CircIn, Ease::CircOut),
        (Ease::BackIn, Ease::BackOut),
        (Ease::ElasticIn, Ease::ElasticOut),
        (Ease::BounceIn, Ease::BounceOut),
    ];

    #[test]
    fn every_ease_hits_endpoints() {
        for ease in Ease::ALL {
            assert!(approx(ease.apply(0.0), 0.0), "{ease:?} at 0");
            assert!(approx(ease.apply(1.0), 1.0), "{ease:?} at 1");
        }
    }

    #[test]
    fn in_out_curves_pass_through_half() {
        for ease in [
            Ease::SineInOut,
            Ease::QuadInOut,
            Ease::CubicInOut,
            Ease::QuartInOut,
            Ease::QuintInOut,
            Ease::ExpoInOut,
            Ease::CircInOut,
            Ease::BackInOut,
            Ease::ElasticInOut,
            Ease::BounceInOut,
        ] {
            assert!(approx(ease.apply(0.5), 0.5), "{ease:?}");
        }
    }

    #[test]
    fn out_mirrors_in() {
        for (ein, eout) in PAIRS {
            for i in 0..=20 {
                let t = i as f32 / 20.0;
                assert!(
                    approx(eout.apply(t), 1.0 - ein.apply(1.0 - t)),
                    "{eout:?} vs {ein:?} at {t}"
                );
            }
        }
    }

    #[test]
    fn in_out_is_point_symmetric() {
        for ease in [
            Ease::SineInOut,
            Ease::QuadInOut,
            Ease::CubicInOut,
            Ease::QuartInOut,
            Ease::QuintInOut,
            Ease::ExpoInOut,
            Ease::CircInOut,
            Ease::BackInOut,
            Ease::ElasticInOut,
            Ease::BounceInOut,
        ] {
            for i in 0..=20 {
                let t = i as f32 / 20.0;
                assert!(
                    approx(ease.apply(t), 1.0 - ease.apply(1.0 - t)),
                    "{ease:?} at {t}"
                );
            }
        }
    }

    #[test]
    fn known_values() {
        assert!(approx(Ease::Linear.apply(0.3), 0.3));
        assert!(approx(Ease::QuadIn.apply(0.5), 0.25));
        assert!(approx(Ease::QuadOut.apply(0.5), 0.75));
        assert!(approx(Ease::CubicIn.apply(0.5), 0.125));
        assert!(approx(Ease::QuartIn.apply(0.5), 0.0625));
        assert!(approx(Ease::QuintIn.apply(0.5), 0.03125));
        assert!(approx(Ease::ExpoIn.apply(0.5), 1.0 / 32.0));
        assert!(approx(
            Ease::SineOut.apply(0.5),
            core::f32::consts::FRAC_1_SQRT_2
        ));
        assert!(approx(Ease::CircOut.apply(0.5), 0.75f32.sdl_sqrt()));
    }

    #[test]
    fn overshooting_curves_leave_unit_range() {
        let range = |e: Ease| {
            (0..=100)
                .map(|i| e.apply(i as f32 / 100.0))
                .fold((<f32>::MAX, <f32>::MIN), |(lo, hi), v| {
                    (lo.min(v), hi.max(v))
                })
        };

        assert!(range(Ease::BackOut).1 > 1.05);
        assert!(range(Ease::BackIn).0 < -0.05);
        assert!(range(Ease::ElasticOut).1 > 1.05);
        assert!(range(Ease::ElasticIn).0 < -0.05);

        for e in [
            Ease::QuadInOut,
            Ease::BounceOut,
            Ease::BounceIn,
            Ease::CircInOut,
        ] {
            let (lo, hi) = range(e);
            assert!(lo >= -EPS && hi <= 1.0 + EPS, "{e:?}");
        }
    }

    #[test]
    fn standard_curves_are_monotonic() {
        for e in [
            Ease::Linear,
            Ease::SineInOut,
            Ease::QuadIn,
            Ease::CubicOut,
            Ease::QuartInOut,
            Ease::QuintOut,
            Ease::ExpoInOut,
            Ease::CircIn,
        ] {
            let mut prev = e.apply(0.0);
            for i in 1..=100 {
                let v = e.apply(i as f32 / 100.0);
                assert!(v >= prev - EPS, "{e:?}");
                prev = v;
            }
        }
    }

    #[test]
    fn bezier_linear_is_identity() {
        let e = Ease::cubic_bezier(0.0, 0.0, 1.0, 1.0);
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            assert!(approx(e.apply(t), t), "{t}");
        }
    }

    #[test]
    fn bezier_matches_quad_shape() {
        let e = Ease::cubic_bezier(1.0 / 3.0, 0.0, 2.0 / 3.0, 1.0 / 3.0);
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            assert!(approx(e.apply(t), t * t), "{t}");
        }
    }

    #[test]
    fn bezier_overshoots_with_y_out_of_range() {
        let e = Ease::cubic_bezier(0.34, 1.56, 0.64, 1.0);
        let peak = (0..=100)
            .map(|i| e.apply(i as f32 / 100.0))
            .fold(0.0f32, |a, b| a.max(b));
        assert!(peak > 1.05);
    }

    #[test]
    fn bezier_clamps_x() {
        assert_eq!(
            Ease::cubic_bezier(-1.0, 0.0, 2.0, 1.0),
            Ease::CubicBezier(0.0, 0.0, 1.0, 1.0)
        );
        let raw = Ease::CubicBezier(-1.0, 0.0, 2.0, 1.0);
        assert!(!raw.apply(0.5).is_nan());
    }

    #[test]
    fn bezier_steep_curve_converges() {
        let e = Ease::cubic_bezier(0.0, 0.0, 0.0, 1.0);
        let mut prev = 0.0;
        for i in 0..=200 {
            let v = e.apply(i as f32 / 200.0);
            assert!(!v.is_nan() && v >= prev - EPS);
            prev = v;
        }
    }

    #[test]
    fn lerp_impls() {
        assert_eq!(2.0f32.lerp(&4.0, 0.5), 3.0);
        assert_eq!(2.0f64.lerp(&4.0, 0.25), 2.5);
        assert_eq!(2.0f32.lerp(&4.0, 1.5), 5.0);
        let v = Lerp::lerp(&Vector2::new(0.0, 10.0), &Vector2::new(10.0, 0.0), 0.5);
        assert_eq!(v, Vector2::new(5.0, 5.0));
        let s = Size::new(10.0f32, 20.0).lerp(&Size::new(20.0, 40.0), 0.5);
        assert_eq!(s, Size::new(15.0, 30.0));
    }

    #[test]
    fn once_reaches_target_exactly_and_finishes() {
        let mut t = Tween::new(0.1f32, 0.7, secs(1.0));
        assert!(!t.is_finished());
        assert!(approx(t.update(0.5), 0.4));
        assert_eq!(t.update(10.0), 0.7);
        assert!(t.is_finished());
        assert_eq!(t.progress(), 1.0);
        assert_eq!(t.value(), 0.7);
    }

    #[test]
    fn starts_at_from() {
        let t = Tween::new(3.0f32, 9.0, secs(1.0));
        assert_eq!(t.value(), 3.0);
        assert_eq!(t.progress(), 0.0);
    }

    #[test]
    fn ease_is_applied() {
        let mut t = Tween::new(0.0f32, 1.0, secs(1.0)).with_ease(Ease::QuadIn);
        assert!(approx(t.update(0.5), 0.25));
    }

    #[test]
    fn loop_wraps() {
        let mut a = Tween::new(0.0f32, 10.0, secs(1.0)).with_repeat(Repeat::Loop);
        let mut b = a;
        assert!(approx(a.update(1.5), b.update(0.5)));
        assert!(!a.is_finished());
    }

    #[test]
    fn ping_pong_mirrors() {
        let mut a = Tween::new(0.0f32, 10.0, secs(1.0)).with_repeat(Repeat::PingPong);
        let mut b = a;
        assert!(approx(a.update(1.25), b.update(0.75)));
        let mut c = a;
        assert!(approx(c.update(0.75), 0.0));
        assert!(!a.is_finished());
    }

    #[test]
    fn long_loops_stay_precise() {
        let mut t = Tween::new(0.0f32, 1.0, secs(1.0)).with_repeat(Repeat::Loop);
        for _ in 0..100_000 {
            t.update(0.25);
        }
        assert!(approx(t.progress(), 0.0) || approx(t.progress(), 1.0));
    }

    #[test]
    fn reset_restarts() {
        let mut t = Tween::new(0.0f32, 1.0, secs(1.0));
        t.update(2.0);
        t.reset();
        assert!(!t.is_finished());
        assert_eq!(t.value(), 0.0);
    }

    #[test]
    fn retarget_is_continuous() {
        let mut t = Tween::new(0.0f32, 10.0, secs(1.0)).with_ease(Ease::CubicInOut);
        let before = t.update(0.3);
        t.retarget(-5.0);
        assert!(approx(t.value(), before));
        assert_eq!(t.update(5.0), -5.0);
    }

    #[test]
    fn zero_duration_is_finished() {
        let mut t = Tween::new(1.0f32, 2.0, Duration::ZERO);
        assert!(t.is_finished());
        assert_eq!(t.update(0.016), 2.0);
        let mut l = Tween::new(1.0f32, 2.0, Duration::ZERO).with_repeat(Repeat::Loop);
        assert!(!l.update(0.016).is_nan());
    }

    #[test]
    fn builders_and_setters() {
        let mut t = Tween::new(0.0f32, 1.0, secs(1.0))
            .with_from(2.0)
            .with_to(4.0)
            .with_duration(secs(2.0))
            .with_ease(Ease::Linear)
            .with_repeat(Repeat::Once);
        assert!(approx(t.update(1.0), 3.0));

        t.reset();
        t.set_from(10.0);
        t.set_to(20.0);
        t.set_duration(secs(4.0));
        t.set_ease(Ease::QuadIn);
        t.set_repeat(Repeat::Once);
        assert!(approx(t.update(2.0), 12.5));
    }
}
