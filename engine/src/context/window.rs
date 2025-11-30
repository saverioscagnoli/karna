use math::Size;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Window(Arc<winit::window::Window>);

impl Window {
    pub(crate) fn from_winit(inner: Arc<winit::window::Window>) -> Self {
        Self(inner)
    }

    #[inline]
    pub fn size(&self) -> Size<u32> {
        self.0.inner_size().into()
    }

    #[inline]
    pub(crate) fn request_redraw(&self) {
        self.0.request_redraw();
    }
}
