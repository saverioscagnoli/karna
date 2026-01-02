use macros::Get;

use crate::profiling::STATS;

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
#[derive(Get)]
pub struct RenderStats {
    #[get(copied)]
    pub(crate) draw_calls: u32,

    #[get(copied)]
    pub(crate) triangles: u32,

    #[get(copied)]
    pub(crate) vertices: u32,

    #[get(copied)]
    pub(crate) indices: u32,

    #[get(copied)]
    pub(crate) instances: u32,

    #[get(copied)]
    pub(crate) instance_writes: u32,

    #[get(copiedj)]
    pub(crate) geometry_buffers: u32,

    #[get(copied)]
    pub(crate) geometry_buffers_size: u32,

    #[get(copied)]
    pub(crate) immediate_draws: u32,

    #[get(copied)]
    pub(crate) pipeline_switches: u32,
}

impl RenderStats {
    /// Only resets values that are meant
    /// to be calculated per-frame
    /// (Draw calls, etc.)
    pub fn reset_frame(&mut self) {
        *self = Self {
            geometry_buffers: self.geometry_buffers,
            geometry_buffers_size: self.geometry_buffers_size,
            ..Self::default()
        }
    }
}

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
pub fn record_pipeline_switches(count: u32) {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.render.pipeline_switches += count;
    });
}

#[inline]
pub fn reset_frame() {
    STATS.with(|stats| {
        let mut stats = stats.borrow_mut();

        stats.render.reset_frame();
    });
}
