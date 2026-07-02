use renderer::Draw;
use renderer::Renderer;

use crate::input::Input;
use crate::time::Time;
use crate::window::Window;

pub struct Context {
    pub window: Window,
    pub time: Time,
    pub input: Input,
    pub render: Option<Renderer>,
}

impl Context {
    pub(crate) fn new(window: Window) -> Self {
        let view = window.size();
        Self {
            window,
            time: Time::new(),
            input: Input::new(),
            render: None,
        }
    }

    pub(crate) fn render(&self) -> &Renderer {
        self.render.as_ref().expect("renderer not initialized")
    }

    pub(crate) fn render_mut(&mut self) -> &mut Renderer {
        self.render.as_mut().expect("renderer not initialized")
    }

    pub(crate) fn init_gpu(&mut self) {
        self.render = Some(Renderer::new(self.window.size()));
    }

    pub(crate) fn as_mut<'a>(&'a mut self) -> ContextRefMut<'a> {
        ContextRefMut {
            window: &mut self.window,
            time: &mut self.time,
            input: &mut self.input,
        }
    }

    pub(crate) fn split<'a>(&'a mut self) -> (ContextRef<'a>, Draw<'a>) {
        let ctx = ContextRef {
            window: &self.window,
            time: &self.time,
            input: &self.input,
        };

        let renderer = self.render.as_mut().expect("renderer not initialized");
        (ctx, Draw::_new(renderer))
    }
}

pub struct ContextRefMut<'a> {
    pub window: &'a mut Window,
    pub time: &'a mut Time,
    pub input: &'a mut Input,
}

pub struct ContextRef<'a> {
    pub window: &'a Window,
    pub time: &'a Time,
    pub input: &'a Input,
}
