use std::time::Duration;
use std::time::Instant;

use logging::info;

pub struct TickCounter<const N: usize> {
    stamps: [Instant; N],
    cursor: usize,
    len: usize,
}

impl<const N: usize> Default for TickCounter<N> {
    fn default() -> Self {
        Self {
            stamps: [Instant::now(); N],
            cursor: 0,
            len: 0,
        }
    }
}

impl<const N: usize> TickCounter<N> {
    pub fn push(&mut self, now: Instant) {
        self.stamps[self.cursor] = now;
        self.cursor = (self.cursor + 1) % N;
        self.len = (self.len + 1).min(N);
    }

    pub fn avg(&self) -> f32 {
        if self.len < 2 {
            return 0.0;
        }

        let newest = self.stamps[(self.cursor + N - 1) % N];
        let oldest = self.stamps[if self.len == N { self.cursor } else { 0 }];
        let span = newest.duration_since(oldest);

        if span.is_zero() {
            return 0.0;
        }

        (self.len - 1) as f32 / span.as_secs_f32()
    }
}

pub struct Clock {
    pub tick_rate: Duration,
    pub accumulator: Duration,
    pub last: Instant,
    pub elapsed: Duration,
    pub ticks: u64,
    pub tps: TickCounter<100>,
    pub dropped: Duration,
    pub scale: f32,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_secs_f32(1.0 / 60.0),
            accumulator: Duration::ZERO,
            last: Instant::now(),
            elapsed: Duration::ZERO,
            ticks: 0,
            tps: TickCounter::default(),
            dropped: Duration::ZERO,
            scale: 1.0,
        }
    }
}

impl Clock {
    pub const MAX_CATCH_UP: u32 = 5;
    pub const MIN_TPS: u32 = 1;
    pub const MAX_TPS: u32 = 1000;

    pub fn advance(&mut self, now: Instant) {
        let raw = now.duration_since(self.last);
        self.last = now;

        let cap = self.tick_rate * Self::MAX_CATCH_UP;
        if raw > cap {
            self.dropped += raw - cap;
        }

        self.accumulator += raw.min(cap).mul_f32(self.scale);
    }

    pub fn should_tick(&self) -> bool {
        self.accumulator >= self.tick_rate
    }

    pub fn consume(&mut self, now: Instant) {
        self.accumulator -= self.tick_rate;
        self.elapsed += self.tick_rate;
        self.ticks += 1;
        self.tps.push(now);
    }

    pub fn next_tick(&self) -> Instant {
        let remaining = self.tick_rate.saturating_sub(self.accumulator);

        if self.scale <= f32::EPSILON {
            return self.last + Duration::from_millis(100); // parked; pacer drives
        }

        self.last + remaining.div_f32(self.scale)
    }

    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.tick_rate.as_secs_f32()
    }

    pub fn set_tps(&mut self, tps: u32) {
        let tps = tps.clamp(Self::MIN_TPS, Self::MAX_TPS);

        self.tick_rate = Duration::from_nanos(1_000_000_000 / tps as u64);
        self.accumulator = self.accumulator.min(self.tick_rate * Self::MAX_CATCH_UP);
        info!("Set target tps to {}.", tps);
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale.max(0.0);
        info!("Set time scale to {}", self.scale);
    }
}
