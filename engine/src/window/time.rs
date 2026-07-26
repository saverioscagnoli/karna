use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use utils::SleepTimer;

pub struct Time {
    start: Instant,
    frame_step: Duration,
    next_frame: Instant,
    last_frame: Instant,
    delta_time: f32,

    tps: u32,
    tick_step: f32,
    tick_count: u32,
    tick_acc: f32,
    tick_timer: f32,
    tick_time: Duration,

    fps: f32,
    fps_sample_size: usize,
    target_fps: u32,
    frame_times: VecDeque<Duration>,
    frame_times_sum: Duration,
}

impl Default for Time {
    fn default() -> Self {
        let now = Instant::now();
        let frame_step = Duration::from_secs_f32(1.0 / 120.0);

        Self {
            start: Instant::now(),
            frame_step,
            next_frame: now + frame_step,
            last_frame: now,
            delta_time: frame_step.as_secs_f32(),
            tps: 0,
            tick_step: 1.0 / 60.0,
            tick_count: 0,
            tick_acc: 0.0,
            tick_timer: 0.0,
            tick_time: Duration::ZERO,
            fps: 0.0,
            fps_sample_size: 100,
            target_fps: 60,
            frame_times: VecDeque::new(),
            frame_times_sum: Duration::ZERO,
        }
    }
}

impl Time {
    pub(crate) fn next_frame(&self) -> Instant {
        self.next_frame
    }

    pub(crate) fn should_tick(&self) -> bool {
        self.tick_acc >= self.tick_step
    }

    pub(crate) fn advance(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32().min(0.333);

        self.last_frame = now;

        self.delta_time = dt;
        self.tick_acc += dt;
        self.tick_timer += dt;

        if self.tick_timer >= 1.0 {
            self.tps = self.tick_count;
            self.tick_count = 0;
            self.tick_timer -= 1.0;
        }
    }

    pub(crate) fn consume(&mut self, start: Instant) {
        self.tick_acc -= self.tick_step;
        self.tick_count += 1;
        self.tick_time = Instant::now() - start;
    }

    pub(crate) fn frame_due(&self, now: Instant) -> bool {
        now >= self.next_frame
    }

    pub(crate) fn schedule_next_frame(&mut self) {
        if self.frame_step.is_zero() {
            self.next_frame = Instant::now();
            return;
        }

        self.next_frame += self.frame_step;
        let now = Instant::now();

        if self.next_frame < now {
            // Fell behind (stall, debugger break, window drag). Don't try to
            // replay the missed frames — that's a death spiral. Just resync.
            self.next_frame = now + self.frame_step;
        }
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
