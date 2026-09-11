use sdl3::window::Window;
use sdl3::window::WindowId;

use crate::event::AppEvent;
use crate::event::Outbox;

pub struct WindowData {
    id: WindowId,
}

impl WindowData {
    pub(crate) fn init(window: &Window) -> Self {
        Self { id: window.id() }
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }
}

pub struct WindowHandle<'a> {
    pub(crate) window_id: WindowId,
    pub(crate) data: &'a mut WindowData,
    pub(crate) outbox: &'a mut Outbox<AppEvent>,
}
