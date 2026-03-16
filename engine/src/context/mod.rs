mod time;
mod window;

use renderer::Draw;
use renderer::Renderer;
pub use time::Time;
pub use window::Window;

pub struct ContextRefMut<'a> {
    pub time: &'a mut Time,
    pub window: &'a mut Window,
}

pub struct ContextRef<'a> {
    pub time: &'a Time,
    pub window: &'a Window,
}

pub struct WindowContext {
    pub time: Time,
    pub window: Window,
    pub render: Renderer,
}

impl WindowContext {
    pub(crate) fn new(window: Window, render: Renderer) -> Self {
        Self {
            time: Time::default(),
            window,
            render,
        }
    }

    pub(crate) fn as_ref_mut<'a>(&'a mut self) -> ContextRefMut<'a> {
        ContextRefMut {
            time: &mut self.time,
            window: &mut self.window,
        }
    }

    pub(crate) fn as_ref<'a>(&'a self) -> ContextRef<'a> {
        ContextRef {
            time: &self.time,
            window: &self.window,
        }
    }

    #[inline]
    pub(crate) fn split<'a>(&'a mut self) -> (ContextRef<'a>, Draw<'a>) {
        let context_ref = ContextRef {
            time: &self.time,
            window: &self.window,
        };

        let draw = Draw::new(&mut self.render);

        (context_ref, draw)
    }
}
