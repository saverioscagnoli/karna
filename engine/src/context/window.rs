use std::sync::Arc;

use macros::Get;

#[derive(Debug)]
#[derive(Get)]
pub struct Window {
    inner: Arc<winit::window::Window>,
}

impl Window {
    pub(crate) fn new(inner: Arc<winit::window::Window>) -> Self {
        Self { inner }
    }

    #[inline]
    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }
}
