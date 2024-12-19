pub struct Time {
    pub(crate) delta: f32,
    pub(crate) render_step: f32,
    pub(crate) tick_step: f32,
    pub(crate) fps: u32,
}

impl Time {
    pub fn new() -> Self {
        Self {
            delta: 0.0,
            tick_step: 1.0 / 60.0,
            render_step: 1.0 / 60.0,
            fps: 0,
        }
    }

    pub fn set_target_fps(&mut self, value: u32) {
        self.render_step = 1.0 / value as f32;
    }

    pub fn set_target_ups(&mut self, value: u32) {
        self.tick_step = 1.0 / value as f32;
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }
}
