use std::fmt::Debug;
use std::ops::Deref;
use std::ops::DerefMut;

use math::Size;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;

use crate::Window;

#[derive(Debug)]
pub struct Monitor {
    inner: MonitorHandle,
}

impl Monitor {
    fn new(inner: MonitorHandle) -> Self {
        Self { inner }
    }

    #[inline]
    pub fn name(&self) -> String {
        self.inner.name().unwrap_or(String::from("unknown monitor"))
    }

    #[inline]
    pub fn size(&self) -> Size<u32> {
        self.inner.size().into()
    }

    #[inline]
    pub fn refresh_rate(&self) -> u32 {
        self.inner.refresh_rate_millihertz().unwrap_or(60_000) / 1000
    }
}

pub struct Monitors {
    window: Window,
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
    pub(crate) fn new(window: Window) -> Self {
        Self {
            window,
            monitors: Vec::new(),
        }
    }

    /// Checks all the monitors and collects them into the wrapper struct.
    /// This method is not directly updating the inner monitors
    /// because it requires an event loop, which is not available in the context.
    ///
    /// So this function must be called when monitor changes, then the vec will be broadcast,
    /// then call [`Monitors::update`] to update the inner monitors.
    #[inline]
    pub(crate) fn collect(event_loop: &ActiveEventLoop) -> Vec<Monitor> {
        event_loop.available_monitors().map(Monitor::new).collect()
    }

    #[inline]
    pub(crate) fn update(&mut self, monitors: Vec<Monitor>) {
        self.monitors = monitors;
    }

    #[inline]
    pub fn current(&self) -> Option<Monitor> {
        self.window.inner().current_monitor().map(Monitor::new)
    }
}
