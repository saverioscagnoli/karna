use std::sync::Arc;
use std::thread::JoinHandle;

use assets::AssetsReader;
use assets::Image;
use crossbeam_channel::Sender;
use logging::error;
use utils::Handle;
use winit::event_loop::EventLoopProxy;
use winit::window::Icon;
use winit::window::WindowId;

use crate::AppEvent;
use crate::UserEvent;
use crate::WindowBuilder;

pub type WinitWindow = winit::window::Window;

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

#[derive(Clone)]
pub struct Window {
    inner: Arc<WinitWindow>,
    assets: AssetsReader,
    proxy: EventLoopProxy<UserEvent>,
}

impl Window {
    pub(crate) fn new(
        inner: WinitWindow,
        assets: AssetsReader,
        proxy: EventLoopProxy<UserEvent>,
    ) -> Self {
        Self {
            inner: Arc::new(inner),
            assets,
            proxy,
        }
    }

    pub(crate) fn winit(&self) -> Arc<WinitWindow> {
        self.inner.clone()
    }

    pub(crate) fn id(&self) -> WindowId {
        self.inner.id()
    }

    #[cfg(target_arch = "wasm32")]
    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    pub fn spawn(&self, b: WindowBuilder) {
        if let Err(e) = self.proxy.send_event(UserEvent::SpawnWindow(b)) {
            error!("Failed to send user event: {}", e);
        }
    }

    pub fn title(&self) -> String {
        self.inner.title()
    }

    pub fn size(&self) -> math::Size<u32> {
        self.inner.inner_size().into()
    }

    pub fn set_icon(&self, image: Handle<Image>) {
        let lock = self.assets.read();
        let image = lock.get_image(image);

        self.inner.set_window_icon(Some(
            Icon::from_rgba(image.data.clone(), image.size.width, image.size.height)
                .expect("Failed to set window icon"),
        ));
    }

    pub fn reset_icon(&self) {
        self.inner.set_window_icon(None);
    }

    pub fn set_custom_cursor<H>(&self, image: Handle<Image>, hotspot: H)
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
