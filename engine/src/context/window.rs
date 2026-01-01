use logging::warn;
use macros::Get;
use math::Size;
use std::sync::Arc;
use winit::window::Fullscreen;

pub type WinitWindow = Arc<winit::window::Window>;

#[derive(Debug, Clone)]
#[derive(Get)]
pub struct Window {
    #[get(ty = &str)]
    label: String,

    #[get(visibility = "pub(crate)")]
    inner: WinitWindow,
}

impl Window {
    pub(crate) fn new<L: Into<String>>(label: L, inner: WinitWindow) -> Self {
        Self {
            label: label.into(),
            inner,
        }
    }

    #[inline]
    pub(crate) fn request_redraw(&self) {
        self.inner.request_redraw();
    }

    #[inline]
    /// Returns the size of the inner window viewport.
    /// (Size of the window without decorations)
    pub fn size(&self) -> Size<u32> {
        self.inner.inner_size().into()
    }

    #[inline]
    /// Returns the width of the inner window viewport.
    /// (Width of the window without decorations)
    pub fn width(&self) -> u32 {
        self.inner.inner_size().width
    }

    #[inline]
    /// Returns the height of the inner window viewport.
    /// (Height of the window without decorations)
    pub fn height(&self) -> u32 {
        self.inner.inner_size().height
    }

    #[inline]
    /// Sets the size of the inner window viewport.
    /// (Size of the window without decorations)
    pub fn set_size<S: Into<Size<u32>>>(&self, size: S) -> bool {
        self.inner.request_inner_size(size.into()).is_some()
    }

    #[inline]
    /// Checks if the window is in windowed mode.
    pub fn is_windowed(&self) -> bool {
        self.inner.fullscreen().is_none()
    }

    #[inline]
    /// Sets the window to windowed mode.
    pub fn set_windowed(&self) {
        self.inner.set_fullscreen(None);
    }

    #[inline]
    /// Checks if the window is in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        self.inner.fullscreen().is_some()
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
                use winit::window::Fullscreen;

                warn!(
                    "Exclusive fullscreen is not supported on wayland. Setting borderless fullscreen instead."
                );
                self.inner
                    .set_fullscreen(Some(Fullscreen::Borderless(None)));
                return;
            }
        }

        let Some(monitor) = self.inner.current_monitor() else {
            warn!(
                "couldnt set fullscreen: window.current_monitor() returned None. Setting borderless fullscreen instead."
            );

            self.inner
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
            return;
        };

        let Some(video_mode) = monitor.video_modes().next() else {
            warn!(
                "couldnt set fullscreen: monitor.video_modes().next() returned None. Setting borderless fullscreen instead."
            );

            self.inner
                .set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
            return;
        };

        self.inner
            .set_fullscreen(Some(Fullscreen::Exclusive(video_mode)));
    }

    #[inline]
    /// Checks if the window is in borderless fullscreen mode.
    pub fn is_borderless_fullscreen(&self) -> bool {
        match self.inner.fullscreen() {
            Some(Fullscreen::Borderless(_)) => true,
            _ => false,
        }
    }

    #[inline]
    /// Sets the window to borderless fullscreen mode.
    ///
    /// **NOTE**: This function will fail silently if the window is not in a valid state.
    pub fn set_borderless_fullscreen(&self) {
        if let Some(monitor) = self.inner.current_monitor() {
            self.inner
                .set_fullscreen(Some(Fullscreen::Borderless(Some(monitor))));
        } else {
            self.inner
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
    }
}
