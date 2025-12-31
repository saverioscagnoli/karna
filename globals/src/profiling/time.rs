use macros::Get;

#[derive(Debug, Clone, Copy)]
#[derive(Default)]
#[derive(Get)]
pub struct TimeStats {
    #[get(copied)]
    pub(crate) elapsed: f32,

    #[get(copied)]
    pub(crate) delta: f32,

    #[get(copied)]
    pub(crate) fps: f32,

    #[get(copied)]
    pub(crate) tps: u32,
}

impl TimeStats {
    #[inline]
    #[doc(hidden)]
    pub fn update(&mut self, delta: f32, fps: f32, tps: u32) {
        self.elapsed += delta;
        self.delta = delta;
        self.fps = fps;
        self.tps = tps;
    }
}
