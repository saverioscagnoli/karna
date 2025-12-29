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
    pub(crate) batches: u32,

    #[get(copied)]
    pub(crate) instances: u32,

    #[get(copied)]
    pub(crate) meshes_drawn: u32,

    #[get(copied)]
    pub(crate) texts_drawn: u32,

    #[get(copied)]
    pub(crate) retained_draws: u32,

    #[get(copied)]
    pub(crate) immediate_draws: u32,

    #[get(copied)]
    pub(crate) peak_draw_calls: u32,

    #[get(copied)]
    pub(crate) peak_triangles: u32,

    #[get(copied)]
    pub(crate) peak_batches: u32,
}

impl RenderStats {
    /// Only resets values that are meant
    /// to be calculated per-frame
    /// (Draw calls, etc.)
    pub fn reset_frame(&mut self) {
        self.peak_draw_calls = self.peak_draw_calls.max(self.draw_calls);
        self.peak_triangles = self.peak_triangles.max(self.triangles);
        self.peak_batches = self.peak_batches.max(self.batches);

        *self = Self {
            peak_draw_calls: self.peak_draw_calls,
            peak_triangles: self.peak_triangles,
            peak_batches: self.peak_batches,
            ..Self::default()
        }
    }
}
