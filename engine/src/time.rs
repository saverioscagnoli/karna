#[derive(Debug, Clone)]
pub struct Time {
    delta: f64,
    elapsed: f64,
    target_ups: u64,
    update_step: f64,
    target_fps: u64,
    render_step: f64,
    fps: f64,
}

impl Time {
    pub(crate) fn new() -> Self {
        Self {
            delta: 0.0,
            elapsed: 0.0,
            target_ups: 60,
            update_step: 1.0 / 60.0,
            target_fps: 60,
            render_step: 1.0 / 60.0,
            fps: 0.0,
        }
    }

    #[inline]
    pub fn update(&mut self, delta: f64) {
        self.delta = delta;
        self.elapsed += delta;

        let current_fps = 1.0 / delta;
        let alpha = 0.5; // Smoothing factor (0.0 = no change, 1.0 = instant)
        self.fps = self.fps * (1.0 - alpha) + current_fps * alpha;
    }

    #[inline]
    pub fn delta(&self) -> f64 {
        self.delta
    }

    #[inline]
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    #[inline]
    pub fn target_fps(&self) -> u64 {
        self.target_fps
    }

    #[inline]
    pub fn fps(&self) -> u64 {
        self.fps.round() as u64
    }

    #[inline]
    pub fn set_target_fps(&mut self, fps: u64) {
        self.target_fps = fps;
        self.render_step = 1.0 / fps as f64;
    }

    #[inline]
    pub fn render_step(&self) -> f64 {
        self.render_step
    }

    #[inline]
    pub fn target_ups(&self) -> u64 {
        self.target_ups
    }

    #[inline]
    pub fn set_target_ups(&mut self, ups: u64) {
        self.target_ups = ups;
        self.update_step = 1.0 / ups as f64;
    }

    #[inline]
    pub fn update_step(&self) -> f64 {
        self.update_step
    }
}
