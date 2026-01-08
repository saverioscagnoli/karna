use crate::state::WinitWindow;
use math::Size;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use winit::{event_loop::ActiveEventLoop, monitor::MonitorHandle};

pub struct Monitors {
    /// Doesn't matter because winitwindow wraps an Arc<winit window>
    /// so we can keep a reference for consistency when getting the current monitor
    window: WinitWindow,

    /// A list of all available monitors
    /// Accessible through deref.
    ///
    /// **NOTE**: This vector is empty until the first draw, so in the `load` function
    /// it will be empty :(
    monitors: Vec<Monitor>,
}

impl Debug for Monitors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Monitors").field(&self.monitors).finish()
    }
}

impl Deref for Monitors {
    type Target = Vec<Monitor>;

    fn deref(&self) -> &Self::Target {
        &self.monitors
    }
}

impl DerefMut for Monitors {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.monitors
    }
}

impl Monitors {
    pub(crate) fn new(window: Arc<winit::window::Window>) -> Self {
        Self {
            window,
            monitors: Vec::new(),
        }
    }

    #[inline]
    /// Checks all the monitors and collects them into the wrapper struct.
    /// This method is not directly updating the inner monitors
    /// because it requires an event loop, which is not available in the context.
    ///
    /// So this function must be called when monitor changes, then the vec will be broadcast,
    /// then call [`Monitors::update`] to update the inner monitors.
    pub(crate) fn collect(event_loop: &ActiveEventLoop) -> Vec<Monitor> {
        event_loop.available_monitors().map(Monitor::new).collect()
    }

    #[inline]
    /// Just sets the monitors.
    ///
    /// See [`Monitors::collect`]
    pub(crate) fn update(&mut self, monitors: Vec<Monitor>) {
        self.monitors = monitors;
    }

    #[inline]
    /// Returns the current monitor that the window is on.
    pub fn current(&self) -> Option<Monitor> {
        self.window.current_monitor().map(Monitor::new)
    }
}

#[derive(Debug)]
pub struct Monitor {
    inner: MonitorHandle,
}

impl Monitor {
    #[inline]
    pub(crate) fn new(inner: MonitorHandle) -> Self {
        Self { inner }
    }

    #[inline]
    /// Returns the name of the monitor in a human-readable format.
    ///
    /// If the name cannot be retrieved, it returns "unknown"
    pub fn name(&self) -> String {
        self.inner.name().unwrap_or(String::from("unknown"))
    }

    #[inline]
    /// Returns the physical size of the monitor in pixels
    pub fn size(&self) -> Size<u32> {
        self.inner.size().into()
    }

    #[inline]
    /// Returns the refresh rate of the monitor in Hz
    ///
    /// If no refresh rate is available, it returns a default of 60 Hz
    pub fn refresh_rate(&self) -> f32 {
        self.inner.refresh_rate_millihertz().unwrap_or(60_000) as f32 / 1000.0
    }
}
