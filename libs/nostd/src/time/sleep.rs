use core::time::Duration;
use sdl3_sys::SDL_DelayNS;
use sdl3_sys::SDL_DelayPrecise;

use crate::time::Instant;

#[inline]
fn as_nanos_u64(d: Duration) -> u64 {
    u64::try_from(d.as_nanos()).unwrap_or(u64::MAX)
}

/// Yields to the OS. Cheap on CPU, imprecise. Use when you don't care
/// about the exact wake time — background threads, backoff, retries.
#[inline]
pub fn sleep(d: Duration) {
    unsafe { SDL_DelayNS(as_nanos_u64(d)) }
}

/// Accurate to well under a millisecond, at the cost of burning CPU
/// on the tail end. Use for frame pacing.
#[inline]
pub fn sleep_precise(d: Duration) {
    unsafe { SDL_DelayPrecise(as_nanos_u64(d)) }
}

/// Sleep until a specific Instant, or return immediately if it's past.
#[inline]
pub fn sleep_precise_until(target: Instant) {
    let now = Instant::now();
    if target > now {
        sleep_precise(target - now);
    }
}
