pub mod context;
pub mod state;

use std::rc::Rc;
use std::sync::mpsc::Sender;

use logging::error;
use utils::WindowId;

pub type SdlWindow = sdl3::video::Window;
pub type SdlEvent = sdl3::event::Event;

pub enum WindowAction {
    SetWindowTitle(WindowId, Rc<str>),
    SetWindowSize(WindowId, math::Size<u32>),
    Present(WindowId),
}

/// Window handle and communicator
/// for window events to the main thread.
///
/// In SDL, the window is not Send nor Sync,
/// they must be created and live on the main thread.
/// The actual game loop in the engine lives
/// in a secondary thread, and this is what
/// permits communication for dispatching window events
pub struct WindowHandle {
    pub(crate) action_sender: Sender<WindowAction>,
    pub(crate) id: WindowId,
    pub(crate) cached_title: Rc<str>,
    pub(crate) cached_size: math::Size<u32>,
}

impl WindowHandle {
    fn send(&self, action: WindowAction) {
        if let Err(e) = self.action_sender.send(action) {
            error!("Failed to send window action: {}", e);
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.cached_title
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        let title: Rc<str> = Rc::from(title.as_ref());

        self.send(WindowAction::SetWindowTitle(self.id, Rc::clone(&title)));
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

        self.send(WindowAction::SetWindowSize(self.id, size));
        self.cached_size = size;
    }

    pub(crate) fn present(&self) {
        self.send(WindowAction::Present(self.id));
    }
}
