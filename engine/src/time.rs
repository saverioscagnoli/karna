use std::time::Instant;

pub struct Time {
    pub(crate) fps_step: f32,
    pub(crate) ups_step: f32,

    pub(crate) t0: Instant,
    pub(crate) t1: Instant,
    pub(crate) t2: f32,

    delta: f32,
    elapsed: f32,
    fps: f32,
}

impl Time {
    const FPS_SMOOTHING_FACTOR: f32 = 0.1;

    pub(crate) fn new() -> Self {
        Self {
            fps_step: 1.0 / 60.0,
            ups_step: 1.0 / 60.0,

            t0: Instant::now(),
            t1: Instant::now(),
            t2: 0.0,

            delta: 0.0,
            elapsed: 0.0,
            fps: 0.0,
        }
    }

    pub(crate) fn update(&mut self, dt: f32) {
        self.delta = dt;
        self.elapsed += dt;

        let fps_now = 1.0 / dt;
        self.fps =
            self.fps * (1.0 - Self::FPS_SMOOTHING_FACTOR) + fps_now * Self::FPS_SMOOTHING_FACTOR
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn frame(&self) -> f32 {
        self.t2
    }

    pub fn fps(&self) -> u32 {
        self.fps.round() as u32
    }

    pub fn set_target_fps(&mut self, t: u32) {
        self.fps_step = 1.0 / t as f32
    }

    pub fn set_target_ups(&mut self, t: u32) {
        self.ups_step = 1.0 / t as f32
    }
}
