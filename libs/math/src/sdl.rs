use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Div;
use core::ops::DivAssign;
use core::ops::Mul;
use core::ops::MulAssign;
use core::ops::Neg;
use core::ops::Sub;
use core::ops::SubAssign;

use sdl3_sys::SDL_acos;
use sdl3_sys::SDL_acosf;
use sdl3_sys::SDL_atan2;
use sdl3_sys::SDL_atan2f;
use sdl3_sys::SDL_cos;
use sdl3_sys::SDL_cosf;
use sdl3_sys::SDL_exp;
use sdl3_sys::SDL_expf;
use sdl3_sys::SDL_fabs;
use sdl3_sys::SDL_fabsf;
use sdl3_sys::SDL_round;
use sdl3_sys::SDL_roundf;
use sdl3_sys::SDL_sin;
use sdl3_sys::SDL_sinf;
use sdl3_sys::SDL_sqrt;
use sdl3_sys::SDL_sqrtf;
use sdl3_sys::SDL_tan;
use sdl3_sys::SDL_tanf;

pub trait SdlFloat:
    Copy
    + PartialOrd
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
{
    const ZERO: Self;
    const ONE: Self;
    const EPSILON: Self;

    fn sdl_round(self) -> Self;
    fn sdl_sqrt(self) -> Self;
    fn sdl_abs(self) -> Self;
    fn sdl_sin(self) -> Self;
    fn sdl_cos(self) -> Self;
    fn sdl_tan(self) -> Self;
    fn sdl_acos(self) -> Self;
    fn sdl_atan2(self, other: Self) -> Self;
    fn sdl_exp(self) -> Self;
    fn sdl_from_f32(v: f32) -> Self;

    #[inline]
    fn sdl_min(self, other: Self) -> Self {
        if self < other { return self } else { other }
    }

    #[inline]
    fn sdl_max(self, other: Self) -> Self {
        if self > other { return self } else { other }
    }

    #[inline]
    fn sdl_clamp(self, lo: Self, hi: Self) -> Self {
        if self < lo {
            lo
        } else if self > hi {
            hi
        } else {
            self
        }
    }
}

impl SdlFloat for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const EPSILON: Self = f32::EPSILON;

    #[inline]
    fn sdl_round(self) -> Self {
        unsafe { SDL_roundf(self) }
    }

    #[inline]
    fn sdl_sqrt(self) -> Self {
        unsafe { SDL_sqrtf(self) }
    }

    #[inline]
    fn sdl_abs(self) -> Self {
        unsafe { SDL_fabsf(self) }
    }
    #[inline]
    fn sdl_sin(self) -> Self {
        unsafe { SDL_sinf(self) }
    }

    #[inline]
    fn sdl_cos(self) -> Self {
        unsafe { SDL_cosf(self) }
    }

    #[inline]
    fn sdl_tan(self) -> Self {
        unsafe { SDL_tanf(self) }
    }

    #[inline]
    fn sdl_acos(self) -> Self {
        unsafe { SDL_acosf(self) }
    }

    #[inline]
    fn sdl_atan2(self, o: Self) -> Self {
        unsafe { SDL_atan2f(self, o) }
    }

    #[inline]
    fn sdl_exp(self) -> Self {
        unsafe { SDL_expf(self) }
    }

    #[inline]
    fn sdl_from_f32(v: f32) -> Self {
        v
    }
}

impl SdlFloat for f64 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
    const EPSILON: Self = f64::EPSILON;

    #[inline]
    fn sdl_round(self) -> Self {
        unsafe { SDL_round(self) }
    }

    #[inline]
    fn sdl_sqrt(self) -> Self {
        unsafe { SDL_sqrt(self) }
    }

    #[inline]
    fn sdl_abs(self) -> Self {
        unsafe { SDL_fabs(self) }
    }

    #[inline]
    fn sdl_sin(self) -> Self {
        unsafe { SDL_sin(self) }
    }

    #[inline]
    fn sdl_cos(self) -> Self {
        unsafe { SDL_cos(self) }
    }

    #[inline]
    fn sdl_tan(self) -> Self {
        unsafe { SDL_tan(self) }
    }

    #[inline]
    fn sdl_acos(self) -> Self {
        unsafe { SDL_acos(self) }
    }

    #[inline]
    fn sdl_atan2(self, o: Self) -> Self {
        unsafe { SDL_atan2(self, o) }
    }

    fn sdl_exp(self) -> Self {
        unsafe { SDL_exp(self) }
    }

    #[inline]
    fn sdl_from_f32(v: f32) -> Self {
        v as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::f32::consts::FRAC_PI_4;
    use core::f32::consts::PI;

    fn approx32(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    fn approx64(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn constants() {
        assert_eq!(<f32 as SdlFloat>::ZERO, 0.0);
        assert_eq!(<f32 as SdlFloat>::ONE, 1.0);
        assert_eq!(<f32 as SdlFloat>::EPSILON, f32::EPSILON);
        assert_eq!(<f64 as SdlFloat>::ZERO, 0.0);
        assert_eq!(<f64 as SdlFloat>::ONE, 1.0);
        assert_eq!(<f64 as SdlFloat>::EPSILON, f64::EPSILON);
    }

    #[test]
    fn round() {
        assert_eq!(2.4f32.sdl_round(), 2.0);
        assert_eq!(2.5f32.sdl_round(), 3.0);
        assert_eq!((-2.5f32).sdl_round(), -3.0);
        assert_eq!(2.5f64.sdl_round(), 3.0);
        assert_eq!((-0.4f64).sdl_round(), 0.0);
    }

    #[test]
    fn sqrt() {
        assert_eq!(9.0f32.sdl_sqrt(), 3.0);
        assert_eq!(2.0f64.sdl_sqrt(), core::f64::consts::SQRT_2);
        assert!((-1.0f32).sdl_sqrt().is_nan());
    }

    #[test]
    fn abs() {
        assert_eq!((-3.5f32).sdl_abs(), 3.5);
        assert_eq!(3.5f32.sdl_abs(), 3.5);
        assert_eq!((-0.0f64).sdl_abs().to_bits(), 0.0f64.to_bits());
    }

    #[test]
    fn trig_f32() {
        assert!(approx32(0.0f32.sdl_sin(), 0.0));
        assert!(approx32((PI / 2.0).sdl_sin(), 1.0));
        assert!(approx32(0.0f32.sdl_cos(), 1.0));
        assert!(approx32(PI.sdl_cos(), -1.0));
        assert!(approx32(FRAC_PI_4.sdl_tan(), 1.0));
        assert!(approx32(1.0f32.sdl_acos(), 0.0));
        assert!(approx32((-1.0f32).sdl_acos(), PI));
    }

    #[test]
    fn trig_f64() {
        use core::f64::consts::PI;
        assert!(approx64((PI / 2.0).sdl_sin(), 1.0));
        assert!(approx64(PI.sdl_cos(), -1.0));
        assert!(approx64((PI / 4.0).sdl_tan(), 1.0));
        assert!(approx64(0.0f64.sdl_acos(), PI / 2.0));
    }

    #[test]
    fn atan2_quadrants() {
        assert!(approx32(1.0f32.sdl_atan2(1.0), FRAC_PI_4));
        assert!(approx32(1.0f32.sdl_atan2(-1.0), 3.0 * FRAC_PI_4));
        assert!(approx32((-1.0f32).sdl_atan2(-1.0), -3.0 * FRAC_PI_4));
        assert!(approx32((-1.0f32).sdl_atan2(1.0), -FRAC_PI_4));
        assert!(approx32(0.0f32.sdl_atan2(1.0), 0.0));
        assert!(approx64(
            1.0f64.sdl_atan2(0.0),
            core::f64::consts::FRAC_PI_2
        ));
    }

    #[test]
    fn exp() {
        assert_eq!(0.0f32.sdl_exp(), 1.0);
        assert!(approx32(1.0f32.sdl_exp(), core::f32::consts::E));
        assert!(approx64(1.0f64.sdl_exp(), core::f64::consts::E));
    }

    #[test]
    fn from_f32() {
        assert_eq!(f32::sdl_from_f32(1.5), 1.5);
        assert_eq!(f64::sdl_from_f32(1.5), 1.5);
    }

    #[test]
    fn min_max() {
        assert_eq!(1.0f32.sdl_min(2.0), 1.0);
        assert_eq!(2.0f32.sdl_min(1.0), 1.0);
        assert_eq!(1.0f64.sdl_max(2.0), 2.0);
        assert_eq!(2.0f64.sdl_max(1.0), 2.0);
        assert_eq!((-0.5f32).sdl_max(-1.5), -0.5);
    }

    #[test]
    fn clamp() {
        assert_eq!(5.0f32.sdl_clamp(0.0, 1.0), 1.0);
        assert_eq!((-5.0f32).sdl_clamp(0.0, 1.0), 0.0);
        assert_eq!(0.5f64.sdl_clamp(0.0, 1.0), 0.5);
        assert_eq!(1.0f32.sdl_clamp(1.0, 1.0), 1.0);
    }
}
