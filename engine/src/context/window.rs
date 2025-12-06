use math::Size;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Window(Arc<winit::window::Window>);

impl Window {
    pub(crate) fn new(inner: Arc<winit::window::Window>) -> Self {
        Self(inner)
    }

    #[inline]
    pub(crate) fn request_redraw(&self) {
        self.0.request_redraw();
    }

    #[inline]
    #[doc(hidden)]
    pub(crate) fn inner(&self) -> &Arc<winit::window::Window> {
        &self.0
    }

    #[inline]
    /// Returns the size of the inner window viewport.
    /// (Size of the window without decorations)
    pub fn size(&self) -> Size<u32> {
        self.0.inner_size().into()
    }

    #[inline]
    /// Returns the width of the inner window viewport.
    /// (Width of the window without decorations)
    pub fn width(&self) -> u32 {
        self.0.inner_size().width
    }

    #[inline]
    /// Returns the height of the inner window viewport.
    /// (Height of the window without decorations)
    pub fn height(&self) -> u32 {
        self.0.inner_size().height
    }
}
