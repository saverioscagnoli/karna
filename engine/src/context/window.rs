use std::sync::Arc;

use assets::AssetServer;
use assets::Image;
use crossbeam_channel::Sender;
use logging::info;
use macros::Get;
use utils::Handle;
use winit::window::Icon;
use winit::window::WindowId;

use crate::events::AppCommand;

#[derive(Get)]
pub struct Window {
    inner: Arc<winit::window::Window>,
    assets: AssetServer,
    cmd_tx: Sender<AppCommand>,
}

impl Window {
    pub(crate) fn new(
        inner: Arc<winit::window::Window>,
        assets: AssetServer,
        cmd_tx: Sender<AppCommand>,
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
    pub fn set_icon(&self, image: Handle<Image>) {
        let guard = self.assets.guard();
        let image = guard.get_image(image);

        info!("Setting a new icon for the window");

        self.inner.set_window_icon(Some(
            Icon::from_rgba(image.rgba.clone(), image.size.width, image.size.height).unwrap(),
        ));
    }

    pub fn id(&self) -> WindowId {
        self.inner.id()
    }

    /// Request setting the cursor from an atlas image handle.
    ///
    /// The actual cursor resource is created on the winit thread.
    pub fn set_cursor_image(&self, image: Handle<Image>, hotspot_x: u16, hotspot_y: u16) {
        info!("Requesting custom cursor change");

        let _ = self.cmd_tx.send(AppCommand::SetCustomCursor {
            window_id: self.inner.id(),
            image,
            hotspot_x,
            hotspot_y,
        });
    }
}
