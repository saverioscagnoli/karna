use std::cell::Cell;
use std::sync::Arc;
use std::thread::JoinHandle;

use assets::AssetsReader;
use assets::Image;
use crossbeam_channel::Sender;
use logging::error;
use utils::Handle;
use winit::event_loop::EventLoopProxy;
use winit::window::CursorGrabMode;
use winit::window::Fullscreen;
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
    cursor_captured: Cell<bool>,
    proxy: EventLoopProxy<UserEvent>,
}

impl Window {
    pub(crate) fn new(
        inner: WinitWindow,
        assets: AssetsReader,
        proxy: EventLoopProxy<UserEvent>,
    ) -> Self {
        inner.set_ime_allowed(true);

        Self {
            inner: Arc::new(inner),
            assets,
            cursor_captured: Cell::new(false),
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

    pub fn close(&self) {
        if let Err(e) = self.proxy.send_event(UserEvent::CloseWindow(self.id())) {
            error!("Failed to send user event: {}", e);
        }
    }

    pub fn title(&self) -> String {
        self.inner.title()
    }

    pub fn size(&self) -> math::Size<u32> {
        self.inner.inner_size().into()
    }

    pub fn fullscreen(&self) -> bool {
        matches!(self.inner.fullscreen(), Some(Fullscreen::Exclusive(_)))
    }

    pub fn set_fullscreen(&self, fullscreen: bool) {
        if fullscreen
            && let Some(monitor) = self.inner.current_monitor()
            && let Some(video_mode) = monitor.video_modes().collect::<Vec<_>>().first().cloned()
        {
            self.inner
                .set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
        } else {
            self.inner.set_fullscreen(None);
        }
    }

    pub fn toggle_fullscreen(&self) {
        if self.fullscreen() {
            self.set_fullscreen(false);
        } else {
            self.set_fullscreen(true);
        }
    }

    pub fn borderless(&self) -> bool {
        matches!(self.inner.fullscreen(), Some(Fullscreen::Borderless(_)))
    }

    pub fn set_borderless(&self, borderless: bool) {
        if borderless {
            self.inner
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        } else {
            self.inner.set_fullscreen(None);
        }
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

    pub fn cursor_captured(&self) -> bool {
        self.cursor_captured.get()
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
