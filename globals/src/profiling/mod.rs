mod memory;
mod render;
mod time;

use crate::profiling::{memory::MemoryStats, time::TimeStats};
use std::cell::RefCell;

pub use render::*;

thread_local! {
    static STATS: RefCell<Statistics> = RefCell::new(Statistics::default());
}

#[inline]
pub fn get_stats() -> Statistics {
    STATS.with(|stats| *stats.borrow())
}

// === Time Stats ===

pub fn update_time(delta: f32, fps: f32, tps: u32) {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.time.update(delta, fps, tps);
    });
}

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub struct Statistics {
    pub render: RenderStats,
    pub time: TimeStats,
    pub mem: MemoryStats,
}
