//use std::cell::Cell;
//use std::sync::Arc;
//use std::sync::mpsc::Sender;
//use std::thread::JoinHandle;
//
//use assets::Image;
//use math::Size;
//use utils::Handle;
//use winit::event_loop::EventLoopProxy;
//use winit::window::CursorGrabMode;
//use winit::window::WindowId;
//
//pub(crate) type WinitWindow = winit::window::Window;
//
//pub struct Window {
//    pub(crate) inner: Arc<WinitWindow>,
//    proxy: EventLoopProxy<UserEvent>,
//    cursor_captured: Cell<bool>,
//}
//
//impl Window {
//    pub(crate) fn new(inner: Arc<WinitWindow>, proxy: EventLoopProxy<UserEvent>) -> Self {
//        Self {
//            inner,
//            proxy,
//            cursor_captured: Cell::new(false),
//        }
//    }
//
//    pub(crate) fn id(&self) -> WindowId {
//        self.inner.id()
//    }
//
//    pub fn size(&self) -> Size<u32> {
//        self.inner.inner_size().into()
//    }
//
//    pub fn width(&self) -> u32 {
//        self.inner.inner_size().width
//    }
//
//    pub fn height(&self) -> u32 {
//        self.inner.inner_size().height
//    }
//
//    pub fn scale_factor(&self) -> f64 {
//        self.inner.scale_factor()
//    }
//
//    pub fn cursor_captured(&self) -> bool {
//        self.cursor_captured.get()
//    }
//
//    pub fn capture_cursor(&self, should: bool) {
//        if should {
//            let _ = self
//                .inner
//                .set_cursor_grab(CursorGrabMode::Locked)
//                .or_else(|_e| self.inner.set_cursor_grab(CursorGrabMode::Confined));
//
//            self.inner.set_cursor_visible(false);
//        } else {
//            let _ = self.inner.set_cursor_grab(CursorGrabMode::None);
//
//            self.inner.set_cursor_visible(true);
//        }
//
//        self.cursor_captured.set(should);
//    }
//
//    pub fn toggle_cursor_capture(&self) {
//        self.capture_cursor(!self.cursor_captured());
//    }
//
//    pub fn set_icon(&self, icon: Handle<Image>) {
//        self.proxy
//            .send_event(UserEvent::SetWindowIcon(self.inner.clone(), icon))
//            .expect("Failed to send user event");
//    }
//
//    pub fn set_custom_cursor(&self, image: Handle<Image>, hotspot_x: u16, hotspot_y: u16) {
//        self.proxy
//            .send_event(UserEvent::SetCursor(
//                self.inner.clone(),
//                image,
//                [hotspot_x, hotspot_y].into(),
//            ))
//            .expect("Failed to send user event");
//    }
//
//    pub(crate) fn request_redraw(&self) {
//        self.inner.request_redraw();
//    }
//}
//
//pub struct WindowHandle {
//    pub sender: Sender<AppEvent>,
//    pub thread: JoinHandle<()>,
//    pub window: Arc<WinitWindow>,
//}

use std::sync::Arc;
use std::thread::JoinHandle;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use gpu::WindowSurface;
use winit::event::WindowEvent;
use winit::window::WindowId;

pub type WinitWindow = winit::window::Window;

pub struct WindowHandle {
    pub thread: JoinHandle<()>,
    pub surface: WindowSurface,
    pub event_tx: Sender<WindowEvent>, // To loop thread
    pub draw_rx: Receiver<()>,         // From loop thread
}

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

    pub fn title(&self) -> String {
        self.inner.title()
    }

    pub fn size(&self) -> math::Size<u32> {
        self.inner.inner_size().into()
    }
}
