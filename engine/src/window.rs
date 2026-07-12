use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::thread::JoinHandle;

use assets::Image;
use crossbeam_channel::Sender;
use logging::error;
use utils::Handle;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowId;

use crate::AppEvent;
use crate::UserEvent;

pub struct WindowHandle {
    /// Native: the thread running this window's render/update loop.
    #[cfg(not(target_arch = "wasm32"))]
    pub thread: JoinHandle<()>,

    /// Web: no threads — the state lives here and is stepped from
    /// `RedrawRequested`.
    #[cfg(target_arch = "wasm32")]
    pub state: crate::state::WindowState,

    pub event_tx: Sender<AppEvent>, // To loop thread

    #[allow(unused)]
    pub window: Window,
}

pub type WinitWindow = winit::window::Window;

#[derive(Clone)]
pub struct Window {
    inner: Arc<WinitWindow>,
    proxy: EventLoopProxy<UserEvent>,
}

impl Window {
    pub(crate) fn new(inner: WinitWindow, proxy: EventLoopProxy<UserEvent>) -> Self {
        Self {
            inner: Arc::new(inner),
            proxy,
        }
    }

    pub(crate) fn winit(&self) -> Arc<WinitWindow> {
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

    pub fn set_custom_cursor<H>(&mut self, image: Handle<Image>, hotspot: H)
    where
        H: Into<math::Vector2<u16>>,
    {
        if let Err(e) = self.proxy.send_event(UserEvent::SetCustomCursor(
            self.inner.clone(),
            image,
            hotspot.into(),
        )) {
            error!("Failed to send user event: {}", e);
        }
    }
}
