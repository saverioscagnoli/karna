mod input;
mod time;
mod window;

use assets::AssetServer;
use assets::AssetServerGuard;
pub use input::Input;
pub use input::KeyState;
use renderer::Draw;
use renderer::Renderer;
pub use time::Time;
pub use window::Window;

pub struct ContextRefMut<'a> {
    pub time: &'a mut Time,
    pub window: &'a mut Window,
    pub input: &'a mut Input,
    pub assets: &'a mut AssetServer,
}

pub struct ContextRef<'a> {
    pub time: &'a Time,
    pub window: &'a Window,
    pub input: &'a Input,
    pub assets: AssetServerGuard<'a>,
}

pub struct WindowContext {
    pub time: Time,
    pub window: Window,
    pub input: Input,
    pub render: Renderer,
    pub assets: AssetServer,
}

impl WindowContext {
    pub(crate) fn new(window: Window, render: Renderer, assets: AssetServer) -> Self {
        Self {
            time: Time::new(),
            window,
            input: Input::new(),
            render,
            assets,
        }
    }

    pub(crate) fn as_ref_mut<'a>(&'a mut self) -> ContextRefMut<'a> {
        ContextRefMut {
            time: &mut self.time,
            window: &mut self.window,
            input: &mut self.input,
            assets: &mut self.assets,
        }
    }

    #[inline]
    pub(crate) fn split<'a>(&'a mut self) -> (ContextRef<'a>, Draw<'a>) {
        let context_ref = ContextRef {
            time: &self.time,
            window: &self.window,
            input: &self.input,
            assets: self.assets.guard(),
        };

        let draw = Draw::new(self.assets.guard(), &mut self.render);

        (context_ref, draw)
    }
}
