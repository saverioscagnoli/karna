use crate::input::Input;
use crate::time::Time;
use crate::window::Window;

pub struct Context {
    pub window: Window,
    pub time: Time,
    pub input: Input,
}

impl Context {
    pub(crate) fn new(window: Window) -> Self {
        Self {
            window,
            time: Time::new(),
            input: Input::new(),
        }
    }

    pub(crate) fn as_mut<'a>(&'a mut self) -> ContextRefMut<'a> {
        ContextRefMut {
            window: &mut self.window,
            time: &mut self.time,
            input: &mut self.input,
        }
    }

    pub(crate) fn as_ref<'a>(&'a self) -> ContextRef<'a> {
        ContextRef {
            window: &self.window,
            time: &self.time,
            input: &self.input,
        }
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
