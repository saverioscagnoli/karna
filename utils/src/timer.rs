use std::time::Duration;

pub struct Timer {
    elapsed: f32,
    duration: f32,
    paused: bool,
}

impl Timer {
    /// Creates a new timer with a set duration in seconds.
    pub fn new(duration: Duration) -> Self {
        Self {
            elapsed: 0.0,
            duration: duration.as_secs_f32(),
            paused: false,
        }
    }

    /// Updates the timer. Call this once per frame with the delta time.
    pub fn tick(&mut self, dt: f32) {
        if !self.paused && !self.is_finished() {
            self.elapsed += dt;
        }
    }

    /// Returns true only on the frame where the timer finishes.
    /// Useful for one-time events.
    pub fn just_finished(&self, dt: f32) -> bool {
        self.elapsed >= self.duration && self.elapsed - dt < self.duration
    }

    /// Returns true once the timer has finished (stays true).
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Returns the progress as a value between 0.0 and 1.0.
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// Resets the timer to start from 0 again.
    #[inline]
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }

    /// Pauses the timer.
    #[inline]
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resumes the timer.
    #[inline]
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Remaining time in seconds.
    #[inline]
    pub fn remaining(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0)
    }
}
