use std::time::Duration;
use std::time::Instant;

use utils::SleepTimer;

pub struct Time {
    frame_step: Duration,
    next_frame: Instant,
    last_frame: Instant,
    delta_time: f32,
    sleep_timer: SleepTimer,
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        let frame_step = Duration::from_secs_f32(1.0 / 60.0);

        Self {
            frame_step,
            next_frame: now + frame_step,
            last_frame: now,
            delta_time: frame_step.as_secs_f32(),
            sleep_timer: SleepTimer::new(),
        }
    }

    pub fn delta(&self) -> f32 {
        self.delta_time
    }

    pub fn wait_for_next_frame(&mut self) {
        self.sleep_timer.sleep_until(self.next_frame);

        let now = Instant::now();
        self.next_frame += self.frame_step;

        // If we've fallen behind (e.g. debugger, long OS preemption),
        // skip missed frames instead of trying to catch up.
        if self.next_frame < now {
            self.next_frame = now + self.frame_step;
        }

        self.delta_time = now.duration_since(self.last_frame).as_secs_f32().min(0.1);
        self.last_frame = now;
    }
}
