use std::sync::atomic::{AtomicU32, Ordering};

use crate::utils::AtomicF32;

pub(crate) static FPS: AtomicU32 = AtomicU32::new(0);
pub(crate) static TPS: AtomicU32 = AtomicU32::new(0);
pub(crate) static DELTA: AtomicF32 = AtomicF32::new(0.0);

#[derive(Debug, Clone, Copy)]
pub struct Time {
    pub(crate) tick_step: f32,
    pub(crate) render_step: f32,
}

impl Time {
    pub(crate) fn new() -> Self {
        Self {
            tick_step: 1.0 / 60.0,
            render_step: 1.0 / 60.0,
        }
    }

    pub fn delta(&self) -> f32 {
        DELTA.load(Ordering::Relaxed)
    }

    pub fn step(&self) -> f32 {
        self.tick_step
    }

    pub fn set_target_tps(&mut self, tps: u32) {
        self.tick_step = 1.0 / tps as f32;
    }

    pub fn set_target_fps(&mut self, fps: u32) {
        self.render_step = 1.0 / fps as f32;
    }

    pub fn tps(&self) -> u32 {
        TPS.load(Ordering::Relaxed)
    }

    pub fn fps(&self) -> u32 {
        FPS.load(Ordering::Relaxed)
    }
}

// unsafe impl Send for Time {}
// unsafe impl Sync for Time {}
