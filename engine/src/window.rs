use std::cell::Cell;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use math::Size;
use winit::window::CursorGrabMode;
use winit::window::WindowId;

use crate::AppEvent;

pub(crate) type WinitWindow = winit::window::Window;

pub struct Window {
    pub(crate) inner: Arc<WinitWindow>,
    cursor_captured: Cell<bool>,
}

impl Window {
    pub(crate) fn new(inner: Arc<WinitWindow>) -> Self {
        Self {
            inner,
            cursor_captured: Cell::new(false),
        }
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

    pub fn cursor_captured(&self) -> bool {
        self.cursor_captured.get()
    }

    pub fn capture_cursor(&self, should: bool) {
        if should {
            let _ = self
                .inner
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_e| self.inner.set_cursor_grab(CursorGrabMode::Confined));

            self.inner.set_cursor_visible(false);
        } else {
            let _ = self.inner.set_cursor_grab(CursorGrabMode::None);

            self.inner.set_cursor_visible(true);
        }

        self.cursor_captured.set(should);
    }

    pub fn toggle_cursor_capture(&self) {
        self.capture_cursor(!self.cursor_captured());
    }

    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }
}

pub struct WindowHandle {
    pub sender: Sender<AppEvent>,
    pub thread: JoinHandle<()>,
    pub window: Arc<WinitWindow>,
}
