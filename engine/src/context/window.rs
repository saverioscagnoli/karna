use math::Size;
use std::sync::Arc;
use traccia::warn;
use winit::window::Fullscreen;

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

    #[inline]
    /// Sets the size of the inner window viewport.
    /// (Size of the window without decorations)
    pub fn set_size<S: Into<Size<u32>>>(&self, size: S) -> bool {
        self.0.request_inner_size(size.into()).is_some()
    }

    #[inline]
    /// Checks if the window is in windowed mode.
    pub fn is_windowed(&self) -> bool {
        self.0.fullscreen().is_none()
    }

    #[inline]
    /// Sets the window to windowed mode.
    pub fn set_windowed(&self) {
        self.0.set_fullscreen(None);
    }

    #[inline]
    /// Checks if the window is in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        self.0.fullscreen().is_some()
    }

    #[inline]
    /// Sets the window to fullscreen mode.
    ///
    /// **NOTE**: If the monitor is not available, or the window is managed by a wayland compositor
    /// such as sway, it will set borderless fullscreen instead.
    pub fn set_fullscreen(&self) {
        #[cfg(target_os = "linux")]
        {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                warn!(
                    "Exclusive fullscreen is not supported on wayland. Setting borderless fullscreen instead."
                );
                self.0.set_fullscreen(Some(Fullscreen::Borderless(None)));
                return;
            }
        }

        let Some(monitor) = self.0.current_monitor() else {
            warn!(
                "couldnt set fullscreen: window.current_monitor() returned None. Setting borderless fullscreen instead."
            );

            self.0.set_fullscreen(Some(Fullscreen::Borderless(None)));
            return;
        };

        let Some(video_mode) = monitor.video_modes().next() else {
            warn!(
                "couldnt set fullscreen: monitor.video_modes().next() returned None. Setting borderless fullscreen instead."
            );

            self.0
                .set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
            return;
        };

        self.0
            .set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
    }

    #[inline]
    /// Checks if the window is in borderless fullscreen mode.
    pub fn is_borderless_fullscreen(&self) -> bool {
        match self.0.fullscreen() {
            Some(Fullscreen::Borderless(_)) => true,
            _ => false,
        }
    }

    #[inline]
    /// Sets the window to borderless fullscreen mode.
    ///
    /// **NOTE**: This function will fail silently if the window is not in a valid state.
    pub fn set_borderless_fullscreen(&self) {
        if let Some(monitor) = self.0.current_monitor() {
            self.0
                .set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
        } else {
            self.0.set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
    }
}
