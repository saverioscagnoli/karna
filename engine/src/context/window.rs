use std::sync::Arc;

use assets::AssetServer;
use assets::Image;
use crossbeam_channel::Sender;
use logging::info;
use logging::warn;
use macros::Get;
use math::Size;
use utils::Handle;
use winit::dpi::PhysicalSize;
use winit::window::Fullscreen;
use winit::window::Icon;

use crate::events::MainCmd;

#[derive(Get)]
#[derive(Clone)]
pub struct Window {
    #[get(visibility = "pub(crate)")]
    inner: Arc<winit::window::Window>,
    assets: AssetServer,
    cmd_tx: Sender<MainCmd>,
}

impl Window {
    pub(crate) fn new(
        inner: Arc<winit::window::Window>,
        assets: AssetServer,
        cmd_tx: Sender<MainCmd>,
    ) -> Self {
        Self {
            inner,
            assets,
            cmd_tx,
        }
    }

    #[inline]
    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    #[inline]
    pub fn title(&self) -> String {
        self.inner.title()
    }

    #[inline]
    pub fn set_title<T: Into<String>>(&self, title: T) {
        self.inner.set_title(&title.into());
    }

    #[inline]
    pub fn size(&self) -> Size<u32> {
        self.inner.inner_size().into()
    }

    #[inline]
    pub fn set_size<S: Into<Size<u32>>>(&self, size: S) {
        let size: Size<u32> = size.into();

        if self
            .inner
            .request_inner_size(PhysicalSize::from(size))
            .is_some()
        {
            warn!("Failed to set size to {:?}: resizing is disallowed", size);
        }
    }

    #[inline]
    pub fn resizable(&self) -> bool {
        self.inner.is_resizable()
    }

    #[inline]
    pub fn set_resizable(&self, resizable: bool) {
        self.inner.set_resizable(resizable);
    }

    #[inline]
    pub fn decorated(&self) -> bool {
        self.inner.is_decorated()
    }

    #[inline]
    pub fn set_decorated(&self, decorated: bool) {
        self.inner.set_decorations(decorated);
    }

    #[inline]
    pub fn visible(&self) -> bool {
        self.inner.is_visible().unwrap_or(false)
    }

    #[inline]
    pub fn set_visible(&self, visible: bool) {
        self.inner.set_visible(visible);
    }

    #[inline]
    pub fn minimized(&self) -> bool {
        self.inner.is_minimized().unwrap_or(false)
    }

    #[inline]
    pub fn set_minimized(&self, minimized: bool) {
        self.inner.set_minimized(minimized);
    }

    #[inline]
    pub fn maximized(&self) -> bool {
        self.inner.is_maximized()
    }

    #[inline]
    pub fn set_maximized(&self, maximized: bool) {
        self.inner.set_maximized(maximized);
    }

    #[inline]
    pub fn fullscreen(&self) -> bool {
        matches!(self.inner.fullscreen(), Some(Fullscreen::Exclusive(_)))
    }

    #[inline]
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

    #[inline]
    pub fn toggle_fullscreen(&self) {
        if self.fullscreen() {
            self.set_fullscreen(false);
        } else {
            self.set_fullscreen(true);
        }
    }

    #[inline]
    pub fn borderless(&self) -> bool {
        matches!(self.inner.fullscreen(), Some(Fullscreen::Borderless(_)))
    }

    #[inline]
    pub fn set_borderless(&self, borderless: bool) {
        if borderless {
            self.inner
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        } else {
            self.inner.set_fullscreen(None);
        }
    }

    #[inline]
    pub fn toggle_borderless(&self) {
        if self.borderless() {
            self.set_borderless(false);
        } else {
            self.set_borderless(true);
        }
    }

    #[inline]
    pub fn set_icon(&self, image: Handle<Image>) {
        let guard = self.assets.guard();
        let image = guard.get_image(image);

        info!("Setting a new icon for the window");

        self.inner.set_window_icon(Some(
            Icon::from_rgba(image.rgba.clone(), image.size.width, image.size.height).unwrap(),
        ));
    }

    /// Request setting the cursor from an atlas image handle.
    ///
    /// The actual cursor resource is created on the winit thread.
    pub fn set_cursor_image(&self, image: Handle<Image>, hotspot_x: u16, hotspot_y: u16) {
        info!("Requesting custom cursor change");

        let _ = self.cmd_tx.send(MainCmd::SetCustomCursor {
            window_id: self.inner.id(),
            image,
            hotspot_x,
            hotspot_y,
        });
    }
}
