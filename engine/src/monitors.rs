use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Arc;

use math::Size;
use winit::event_loop::ActiveEventLoop;
use winit::monitor::MonitorHandle;

use crate::window::WinitWindow;

pub struct Monitor {
    handle: MonitorHandle,
}

impl Monitor {
    pub(crate) fn new(handle: MonitorHandle) -> Self {
        Self { handle }
    }

    pub fn name(&self) -> String {
        self.handle.name().unwrap_or("unknown monitor".to_owned())
    }

    pub fn size(&self) -> Size<u32> {
        self.handle.size().into()
    }

    pub fn refresh_rate(&self) -> u32 {
        (self.handle.refresh_rate_millihertz().unwrap_or(60_000) as f32 / 1000.0).round() as u32
    }
}

pub struct Monitors {
    window: Arc<WinitWindow>,

    pub(crate) monitors: Vec<Monitor>,
}

impl Monitors {
    /// It will automatically query monitors on startup
    pub(crate) fn new(window: Arc<WinitWindow>, initial: Vec<Monitor>) -> Self {
        Self {
            window,
            monitors: initial,
        }
    }

    /// This method must be called on the main thread, hence why it needs
    /// the event_loop as the argument. It does not update the monitors state
    /// intentionally, because in winit you must query the monitor on the event loop thread
    pub(crate) fn collect(event_loop: &ActiveEventLoop) -> Vec<Monitor> {
        event_loop.available_monitors().map(Monitor::new).collect()
    }

    pub fn current(&self) -> Option<Monitor> {
        self.window.current_monitor().map(Monitor::new)
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
