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
