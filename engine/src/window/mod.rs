pub mod context;
pub mod state;
pub mod time;

use utils::WindowId;

pub type SdlWindow = sdl3::video::Window;
pub type SdlEvent = sdl3::event::Event;
pub type SdlWindowEvent = sdl3::event::WindowEvent;

pub struct Window {
    pub(crate) inner: SdlWindow,
}

impl Window {
    pub(crate) fn wrap(inner: SdlWindow) -> Self {
        Self { inner }
    }

    pub fn id(&self) -> WindowId {
        self.inner.id()
    }

    pub fn title(&self) -> &str {
        self.inner.title()
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        let _ = self.inner.set_title(title.as_ref());
    }
}
