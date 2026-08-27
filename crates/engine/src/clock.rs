use std::time::Duration;
use std::time::Instant;

use logging::debug;

use crate::config::config;

pub struct Clock {
    pub tick_rate: Duration,
    pub accumulator: Duration,
    pub last: Instant,
    pub elapsed: Duration,
    pub droppped: Duration,
    pub scale: f32,
}

impl Default for Clock {
    fn default() -> Self {
        let config = config();

        Self {
            tick_rate: Duration::from_secs_f32(1.0 / config.time.target_tps as f32),
            accumulator: Duration::ZERO,
            last: Instant::now(),
            elapsed: Duration::ZERO,
            droppped: Duration::ZERO,
            scale: 1.0,
        }
    }
}

impl Clock {
    pub fn set_target_tps(&mut self, t: u32) {
        debug!("Set target tps to {}", t);
        self.tick_rate = Duration::from_secs_f32(1.0 / t as f32);
    }

    pub fn advance(&mut self, now: Instant) {
        let dt = now.duration_since(self.last);
        let config = config();
        let cap = self.tick_rate * config.time.max_tick_catchup;

        self.last = now;

        if dt > cap {
            self.droppped += dt - cap;
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
