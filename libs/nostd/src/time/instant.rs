use core::ops::Add;
use core::ops::AddAssign;
use core::ops::Sub;
use core::ops::SubAssign;
use core::sync::atomic::AtomicU64;
use core::sync::atomic::Ordering::Relaxed;
use core::time::Duration;

use sdl3_sys::SDL_GetPerformanceCounter;
use sdl3_sys::SDL_GetPerformanceFrequency;

static FREQ: AtomicU64 = AtomicU64::new(0);

#[inline]
fn freq() -> u64 {
    let cached = FREQ.load(Relaxed);
    if cached != 0 {
        return cached;
    }

    let f = unsafe { SDL_GetPerformanceFrequency() };
    debug_assert!(f != 0, "SDL_GetPerformanceFrequency returned 0");
    let f = if f == 0 { 1_000_000_000 } else { f };

    FREQ.store(f, Relaxed);
    f
}

#[inline]
fn ticks_to_duration(ticks: u64) -> Duration {
    let f = freq();
    let secs = ticks / f;
    let rem = ticks % f;
    let nanos = (rem * 1_000_000_000) / f;

    Duration::new(secs, nanos as u32)
}

#[inline]
fn duration_to_ticks(d: Duration) -> Option<u64> {
    let f = freq();
    let whole = d.as_secs().checked_mul(f)?;
    let sub = (u64::from(d.subsec_nanos()) * f) / 1_000_000_000;

    whole.checked_add(sub)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(u64);

impl Instant {
    #[inline]
    pub fn now() -> Self {
        Self(unsafe { SDL_GetPerformanceCounter() })
    }

    #[inline]
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        ticks_to_duration(self.0.saturating_sub(earlier.0))
    }

    #[inline]
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        self.saturating_duration_since(earlier)
    }

    #[inline]
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.0.checked_sub(earlier.0).map(ticks_to_duration)
    }

    #[inline]
    pub fn checked_add(&self, d: Duration) -> Option<Instant> {
        duration_to_ticks(d)
            .and_then(|t| self.0.checked_add(t))
            .map(Instant)
    }

    #[inline]
    pub fn checked_sub(&self, d: Duration) -> Option<Instant> {
        duration_to_ticks(d)
            .and_then(|t| self.0.checked_add(t))
            .map(Instant)
    }

    #[inline]
    pub fn elapsed(&self) -> Duration {
        Instant::now().duration_since(*self)
    }

    #[inline]
    pub fn raw(&self) -> u64 {
        self.0
    }
}

impl Sub for Instant {
    type Output = Duration;
    #[inline]
    fn sub(self, rhs: Instant) -> Duration {
        self.saturating_duration_since(rhs)
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;
    #[inline]
    fn add(self, d: Duration) -> Instant {
        self.checked_add(d)
            .expect("overflow adding Duration to Instant")
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;
    #[inline]
    fn sub(self, d: Duration) -> Instant {
        self.checked_sub(d)
            .expect("overflow subtracting Duration from Instant")
    }
}

impl AddAssign<Duration> for Instant {
    #[inline]
    fn add_assign(&mut self, d: Duration) {
        *self = *self + d;
    }
}

impl SubAssign<Duration> for Instant {
    #[inline]
    fn sub_assign(&mut self, d: Duration) {
        *self = *self - d;
    }
}
