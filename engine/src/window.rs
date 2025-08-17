use std::sync::Arc;

use math::Size;

#[derive(Debug, Clone)]
pub struct Window {
    inner: Arc<winit::window::Window>,
}

impl Window {
    pub(crate) fn new(inner: Arc<winit::window::Window>) -> Self {
        Self { inner }
    }

    pub fn size(&self) -> Size<u32> {
        self.inner.inner_size().into()
    }

    pub fn request_redraw(&self) {
        self.inner.request_redraw();
    }
}
