pub mod context;
pub mod input;
pub mod resources;
pub mod time;

use std::sync::Arc;
use std::sync::mpsc::Sender;

use logging::error;
use utils::WindowId;

use crate::render::Renderer;
use crate::render::packet::FramePacket;

pub type SdlWindow = sdl3::video::Window;
pub type SdlEvent = sdl3::event::Event;
pub type SdlWindowEvent = sdl3::event::WindowEvent;

/// Window wrapper over sdl windows.
///
/// The user never interacts with an instance
/// of this struct. Its purpose is to be stored
/// onto the main thread, because sdl windows must
/// be created and destroyed there
pub struct WindowSlot {
    pub inner: SdlWindow,
    pub renderer: Renderer,
    pub packet: FramePacket,
}

pub enum WindowAction {
    Center,
    SetPosition(math::Vector2<i32>),
    SetResizable(bool),
    SetSize(math::Size<u32>),
    SetTitle(Arc<str>),
}

pub struct WindowRequest {
    pub window_id: WindowId,
    pub action: WindowAction,
}

pub struct FrameSubmission {
    pub window_id: WindowId,
    pub packet: FramePacket,
}

/// Window handle and communicator
/// for window events to the main thread.
///
/// In SDL, the window is not Send nor Sync,
/// they must be created and live on the main thread.
/// The actual game loop in the engine lives
/// in a secondary thread, and this is what
/// permits communication for dispatching window events
#[derive(Debug)]
pub struct WindowHandle {
    pub(crate) action_sender: Sender<WindowRequest>,
    pub(crate) frame_sender: Sender<FrameSubmission>,
    pub(crate) id: WindowId,
    pub(crate) position: math::Vector2<i32>,
    pub(crate) resizable: bool,
    pub(crate) cached_size: math::Size<u32>,
    pub(crate) cached_title: Arc<str>,
}

impl WindowHandle {
    fn send(&self, action: WindowAction) {
        if let Err(e) = self.action_sender.send(WindowRequest {
            window_id: self.id,
            action,
        }) {
            error!("Failed to send window action: {}", e);
        }
    }

    pub(crate) fn submit(&self, packet: FramePacket) {
        if let Err(e) = self.frame_sender.send(FrameSubmission {
            window_id: self.id,
            packet,
        }) {
            error!("Failed to submit frame: {}", e);
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn center(&self) {
        self.send(WindowAction::Center);
    }

    pub fn position(&self) -> math::Vector2<i32> {
        self.position
    }

    pub fn set_position<P>(&mut self, pos: P)
    where
        P: Into<math::Vector2<i32>>,
    {
        let pos: math::Vector2<i32> = pos.into();
        self.send(WindowAction::SetPosition(pos));
    }

    pub fn is_resizable(&self) -> bool {
        self.resizable
    }

    pub fn set_resizable(&mut self, v: bool) {
        self.send(WindowAction::SetResizable(v));
        self.resizable = v;
    }

    pub fn toggle_resizable(&mut self) {
        self.resizable = !self.resizable;
        self.send(WindowAction::SetResizable(self.resizable));
    }

    pub fn title(&self) -> &str {
        &self.cached_title
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        let title: Arc<str> = Arc::from(title.as_ref());

        self.send(WindowAction::SetTitle(Arc::clone(&title)));
        self.cached_title = title;
    }

    pub fn size(&self) -> math::Size<u32> {
        self.cached_size
    }

    pub fn set_size<T>(&mut self, size: T)
    where
        T: Into<math::Size<u32>>,
    {
        let size: math::Size<u32> = size.into();

        self.send(WindowAction::SetSize(size));
        self.cached_size = size;
    }
}
