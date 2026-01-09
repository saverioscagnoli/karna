use crate::Monitor;
use crossbeam_channel::Sender;
use std::thread::JoinHandle;
use winit::event::{DeviceEvent, WindowEvent};

pub enum WindowMessage {
    Close,
    MonitorsChanged(Vec<Monitor>),
    StartFrame,
    WinitEvent(WindowEvent),
    DeviceEvent(DeviceEvent),
}

pub struct WindowHandle {
    pub sender: Sender<WindowMessage>,
    pub thread: JoinHandle<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LoopState {
    Accumulate,
    Render,
    Exit,
}
