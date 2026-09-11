use core::time::Duration;

use nostd::time::Instant;
use traccia::debug;

pub struct Clock {
    pub tick_rate: Duration,
    pub accumulator: Duration,
    pub last: Instant,
    pub elapsed: Duration,
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
            dropped: Duration::ZERO,
            scale: 1.0,
        }
    }
}

impl Clock {
    pub fn set_target_tps(&mut self, t: u32) {
        debug!("Set target ticks per second to {}", t);
        self.tick_rate = Duration::from_secs_f64(1.0 / t as f64);
    }

    pub fn advance(&mut self, now: Instant) {
        let dt = now.duration_since(self.last);
        let cap = self.tick_rate * 5;

        self.last = now;

        if dt > cap {
            self.dropped += dt - cap;
        }

        self.accumulator += dt.min(cap).mul_f32(self.scale);
    }

    pub fn should_tick(&self) -> bool {
        self.accumulator >= self.tick_rate
    }

    pub fn consume(&mut self) {
        self.accumulator -= self.tick_rate;
        self.elapsed += self.tick_rate;
    }

    pub fn next_tick(&self) -> Instant {
        let remaining = self.tick_rate.saturating_sub(self.accumulator);

        if self.scale <= f32::EPSILON {
            return self.last + Duration::from_millis(100);
        }

        self.last + remaining.div_f32(self.scale)
    }

    pub fn alpha(&self) -> f32 {
        self.accumulator.as_secs_f32() / self.tick_rate.as_secs_f32()
    }
}
