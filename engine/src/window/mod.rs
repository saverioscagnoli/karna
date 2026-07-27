pub mod clock;
pub mod context;
pub mod input;
pub mod pacer;
pub mod resources;
pub mod state;
pub mod time;

use sdl3::sys::video::SDL_SetWindowResizable;
use utils::WindowId;

pub type SdlWindow = sdl3::video::Window;

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

    pub fn set_resizable(&mut self, v: bool) {
        unsafe {
            SDL_SetWindowResizable(self.inner.raw(), v);
        }
    }
}
