use std::time::{Duration, Instant};

/// Struct that holds and computes every bits of
/// information about simulation timings
#[derive(Debug, Clone)]
pub struct Time {
    start: Instant,
    delta_time: f32,
    last_tick: Instant,

    tps: u32,
    tick_step: f32,
    tick_count: u32,
    tick_acc: f32,
    tick_timer: f32,
    tick_time: Duration,
}

impl Time {
    pub(crate) fn new() -> Self {
        Self {
            start: Instant::now(),
            delta_time: 0.0,
            last_tick: Instant::now(),
            tps: 0,
            tick_step: 1.0 / 60.0,
            tick_count: 0,
            tick_acc: 0.0,
            tick_timer: 0.0,
            tick_time: Duration::ZERO,
        }
    }

    pub(crate) fn advance(&mut self) {
        let now = Instant::now();
        let dt = (now - self.last_tick).as_secs_f32().min(0.25);
        self.last_tick = now;

        self.delta_time = dt;
        self.tick_acc += dt;
        self.tick_timer += dt;

        if self.tick_timer >= 1.0 {
            self.tps = self.tick_count;
            self.tick_count = 0;
            self.tick_timer -= 1.0;
        }
    }

    pub(crate) fn should_tick(&self) -> bool {
        self.tick_acc >= self.tick_step
    }

    pub(crate) fn consume(&mut self, tick_start: Instant) {
        self.tick_acc -= self.tick_step;
        self.tick_count += 1;
        self.tick_time = Instant::now() - tick_start;
    }

    pub(crate) fn until_next_tick(&self) -> Duration {
        let remaining = (self.tick_step - self.tick_acc).max(0.0);
        Duration::from_secs_f32(remaining)
    }

    pub(crate) fn drop_backlog(&mut self) {
        self.tick_acc = 0.0;
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn delta(&self) -> f32 {
        self.delta_time
    }

    pub fn fixed_delta(&self) -> f32 {
        self.tick_step
    }

    pub fn tps(&self) -> u32 {
        self.tps
    }
}
