mod memory;
mod render;
mod time;

use crate::profiling::{memory::MemoryStats, render::RenderStats, time::TimeStats};
use std::cell::RefCell;

thread_local! {
    static STATS: RefCell<Statistics> = RefCell::new(Statistics::default());
}

#[inline]
pub fn get_stats() -> Statistics {
    STATS.with(|stats| *stats.borrow())
}

// === Render Stats

#[inline]
pub fn record_draw_call(vertices_n: u32, indices_n: u32) {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.render.draw_calls += 1;
        stats.render.indices += indices_n;
        stats.render.vertices += vertices_n;
    });
}

#[inline]
pub fn record_instance_writes(count: u32) {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.render.instance_writes += count;
    });
}

#[inline]
pub fn record_geometry_buffer(count: u32) {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.render.geometry_buffers += count;
    });
}

#[inline]
pub fn record_geometry_buffers_size(size: u32) {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.render.geometry_buffers_size += size;
    });
}

#[inline]
pub fn record_triangles(index_n: u32) {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.render.triangles += index_n / 3;
    });
}

#[inline]
pub fn reset_frame() {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.render.reset_frame();
    });
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
