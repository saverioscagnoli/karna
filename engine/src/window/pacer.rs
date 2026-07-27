use std::time::Duration;
use std::time::Instant;

use logging::info;

pub struct FpsCounter<const N: usize> {
    samples: [Duration; N],
    cursor: usize,
    len: usize,
    sum: Duration,
}

impl<const N: usize> Default for FpsCounter<N> {
    fn default() -> Self {
        Self {
            samples: [Duration::ZERO; N],
            cursor: 0,
            len: 0,
            sum: Duration::ZERO,
        }
    }
}

impl<const N: usize> FpsCounter<N> {
    pub fn push(&mut self, dt: Duration) {
        if self.len == N {
            self.sum -= self.samples[self.cursor];
        } else {
            self.len += 1;
        }

        self.samples[self.cursor] = dt;
        self.sum += dt;
        self.cursor = (self.cursor + 1) % N;
    }

    pub fn avg(&self) -> f32 {
        if self.sum.is_zero() {
            return 0.0;
        }

        self.len as f32 / self.sum.as_secs_f32()
    }

    pub fn avg_dt(&self) -> Duration {
        self.sum
            .checked_div(self.len as u32)
            .unwrap_or(Duration::ZERO)
    }
}

pub struct FramePacer {
    pub interval: Duration,
    pub next_frame: Instant,
    pub last_frame: Instant,
    pub delta: Duration,
    pub fps: FpsCounter<100>,
}

impl Default for FramePacer {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs_f32(1.0 / 60.0),
            next_frame: Instant::now(),
            last_frame: Instant::now(),
            delta: Duration::from_secs_f32(1.0 / 60.0),
            fps: FpsCounter::default(),
        }
    }
}

impl FramePacer {
    pub const MIN_FPS: u32 = 1;
    pub const MAX_FPS: u32 = u32::MAX;

    pub fn due(&self, now: Instant) -> bool {
        now >= self.next_frame
    }

    pub fn record(&mut self, now: Instant) {
        self.delta = now.duration_since(self.last_frame);
        self.last_frame = now;
        self.fps.push(self.delta);
    }

    pub fn schedule(&mut self, now: Instant) {
        self.next_frame += self.interval;
        if self.next_frame < now {
            self.next_frame = now + self.interval;
        }
    }

    pub fn set_fps(&mut self, fps: u32) {
        let fps = fps.clamp(Self::MIN_FPS, Self::MAX_FPS);

        self.interval = Duration::from_nanos(1_000_000_000 / fps as u64);
        self.next_frame = self.last_frame + self.interval;
        info!("Set target fps to {}.", fps);
    }
}
