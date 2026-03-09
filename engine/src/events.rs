use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, Sender};
use winit::event::{DeviceEvent, WindowEvent};

/// Handle used for sending events to the appropriate window,
/// since winit handles all window events in a single `window_event`
///
/// Each window has its own thread
pub struct WindowHandle {
    pub event_tx: Sender<WindowEvent>,
    pub thread: JoinHandle<()>,
}

impl WindowHandle {
    pub fn new(event_tx: Sender<WindowEvent>, thread: JoinHandle<()>) -> Self {
        Self { event_tx, thread }
    }
}

/// General app event handlers, all events that should be streamed to all windows
/// go here, as an example, since input device events are general, such as mouse delta,
/// they should be available to all windows
pub struct EventHandler {
    pub device_tx: Sender<DeviceEvent>,
    pub device_rx: Receiver<DeviceEvent>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (device_tx, device_rx) = crossbeam_channel::unbounded();

        Self {
            device_tx,
            device_rx,
        }
    }
}
