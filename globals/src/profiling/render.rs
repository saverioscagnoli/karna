use macros::Get;

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
