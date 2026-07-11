use std::sync::Arc;
use std::thread::JoinHandle;

use crossbeam_channel::Sender;
use winit::event::WindowEvent;
use winit::window::WindowId;

pub type WinitWindow = winit::window::Window;

pub struct WindowHandle {
    pub thread: JoinHandle<()>,
    pub event_tx: Sender<WindowEvent>, // To loop thread
    pub window: Window,
}

#[derive(Clone)]
pub struct Window {
    inner: Arc<WinitWindow>,
}

impl Window {
    pub(crate) fn new(inner: WinitWindow) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    pub(crate) fn winit_handle(&self) -> Arc<WinitWindow> {
        self.inner.clone()
    }

    pub(crate) fn id(&self) -> WindowId {
        self.inner.id()
    }

    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    pub fn title(&self) -> String {
        self.inner.title()
    }

    pub fn size(&self) -> math::Size<u32> {
        self.inner.inner_size().into()
    }
}
