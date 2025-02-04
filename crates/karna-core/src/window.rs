pub struct Window {
    pub(crate) inner: sdl3::video::Window,
}

impl Window {
    pub(crate) fn new(inner: sdl3::video::Window) -> Self {
        Self { inner }
    }

    pub(crate) fn swap_buffers(&self) {
        self.inner.gl_swap_window();
    }
}
