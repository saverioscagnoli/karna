use std::time::Duration;
use std::time::Instant;

use logging::debug;

use crate::config::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaceMode {
    /// The display paces the frame
    Display,
    /// Pace frames manually with a spin sleeper
    Fixed,
}

pub struct FramePacer {
    pub mode: PaceMode,
    pub frame_rate: Duration,
    pub last_frame: Instant,
    pub next_frame: Instant,
    pub delta: Duration,
}

impl FramePacer {
    pub fn new(mode: PaceMode) -> Self {
        let conf = config();
        let rate = Duration::from_secs_f32(1.0 / conf.target_fps as f32);

        Self {
            mode,
            frame_rate: rate,
            last_frame: Instant::now(),
            next_frame: Instant::now(),
            delta: rate,
        }
    }

    pub fn set_target_fps(&mut self, t: u32) {
        debug!("Set target fps to {}", t);
        self.frame_rate = Duration::from_secs_f32(1.0 / t as f32);
    }

    pub fn due(&self, now: Instant) -> bool {
        match self.mode {
            PaceMode::Display => true,
            PaceMode::Fixed => now >= self.next_frame,
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        match self.mode {
            PaceMode::Display => None,
            PaceMode::Fixed => Some(self.next_frame),
        }
    }

    pub fn idle_backoff(&self) -> Duration {
        self.frame_rate
    }

    pub fn record(&mut self, now: Instant) {
        self.delta = now.duration_since(self.last_frame);
        self.last_frame = now;

        if self.mode == PaceMode::Fixed {
            self.next_frame += self.frame_rate;

            // Fell behind
            if self.next_frame < now {
                self.next_frame = now + self.frame_rate
            }
        }
    }
}
