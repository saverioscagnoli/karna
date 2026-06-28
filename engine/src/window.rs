use std::sync::Arc;

use math::Size;

pub(crate) type WinitWindow = winit::window::Window;

pub struct Window {
    inner: Arc<WinitWindow>,
}

impl Window {
    pub fn size(&self) -> Size<u32> {
        self.inner.inner_size().into()
    }

    pub fn width(&self) -> u32 {
        self.inner.inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.inner.inner_size().height
    }

    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }
}
