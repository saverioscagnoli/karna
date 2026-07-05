use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use math::Size;
use winit::window::WindowId;

use crate::AppEvent;

pub(crate) type WinitWindow = winit::window::Window;

pub struct Window {
    pub(crate) inner: Arc<WinitWindow>,
}

impl Window {
    pub(crate) fn new(inner: Arc<WinitWindow>) -> Self {
        Self { inner }
    }

    pub(crate) fn id(&self) -> WindowId {
        self.inner.id()
    }

    pub fn size(&self) -> Size<u32> {
        self.inner.inner_size().into()
    }

    pub fn width(&self) -> u32 {
        self.inner.inner_size().width
    }

    pub fn height(&self) -> u32 {
        self.inner.inner_size().height
    }

    pub fn scale_factor(&self) -> f64 {
        self.inner.scale_factor()
    }

    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }
}

pub struct WindowHandle {
    pub sender: Sender<AppEvent>,
    pub thread: JoinHandle<()>,
}
