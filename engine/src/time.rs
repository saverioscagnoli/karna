use std::time::{Duration, Instant};

pub struct Time {
    pub(crate) fps_step: Duration,
    pub(crate) ups_step: Duration,

    pub(crate) last_frame: Instant,
    pub(crate) this_frame: Instant,
    pub(crate) frame_time: Duration,
    pub(crate) update_time: Duration,
    pub(crate) update_accum: Duration,

    delta: Duration,
    elapsed: Duration,
    fps: f32,
    ups: u32,
    ups_count: u32,
    ups_timer: f32,
}

impl Time {
    const FPS_SMOOTHING_FACTOR: f32 = 0.1;

    pub(crate) fn new() -> Self {
        Self {
            fps_step: Duration::from_secs_f32(1.0 / 60.0),
            ups_step: Duration::from_secs_f32(1.0 / 60.0),

            last_frame: Instant::now(),
            this_frame: Instant::now(),
            frame_time: Duration::ZERO,
            update_time: Duration::ZERO,
            update_accum: Duration::ZERO,

            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
            fps: 0.0,
            ups: 0,
            ups_count: 0,
            ups_timer: 0.0,
        }
    }

    pub(crate) fn tick(&mut self, dt: Duration) {
        self.delta = dt;
        self.elapsed += dt;

        let dt = dt.as_secs_f32();

        self.ups_timer += dt;

        let fps_now = 1.0 / dt;
        self.fps =
            self.fps * (1.0 - Self::FPS_SMOOTHING_FACTOR) + fps_now * Self::FPS_SMOOTHING_FACTOR;

        if self.ups_timer >= 1.0 {
            self.ups = self.ups_count;
            self.ups_count = 0;
            self.ups_timer = 0.0;
        }
    }

    pub(crate) fn tick_fixed(&mut self, update_start: Instant) {
        self.ups_count += 1;
        self.update_time = Instant::now() - update_start;
    }

    pub fn delta(&self) -> Duration {
        self.delta
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn frame(&self) -> Duration {
        self.frame_time
    }

    pub fn update(&self) -> Duration {
        self.update_time
    }

    pub fn fps(&self) -> u32 {
        self.fps.round() as u32
    }

    pub fn ups(&self) -> u32 {
        self.ups
    }
    pub fn set_target_fps(&mut self, t: u32) {
        self.fps_step = Duration::from_secs_f32(1.0 / t as f32)
    }

    pub fn set_target_ups(&mut self, t: u32) {
        self.ups_step = Duration::from_secs_f32(1.0 / t as f32)
    }
}
