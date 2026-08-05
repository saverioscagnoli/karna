use crate::window::WindowHandle;
use crate::window::time::Time;

pub struct UserContext {
    pub window: WindowHandle,
}

pub struct LoadContext<'a> {
    pub window: &'a mut WindowHandle,
    pub time: &'a mut Time,
}

pub struct UpdateContext<'a> {
    pub window: &'a mut WindowHandle,
    pub time: &'a mut Time,
}

pub struct DrawContext<'a> {
    pub window: &'a mut WindowHandle,
    pub time: &'a mut Time,
}

impl UserContext {
    pub fn load<'a>(&'a mut self, time: &'a mut Time) -> LoadContext<'a> {
        LoadContext {
            window: &mut self.window,
            time,
        }
    }

    pub fn update<'a>(&'a mut self, time: &'a mut Time) -> UpdateContext<'a> {
        UpdateContext {
            window: &mut self.window,
            time,
        }
    }

    pub fn draw<'a>(&'a mut self, time: &'a mut Time) -> DrawContext<'a> {
        DrawContext {
            window: &mut self.window,
            time,
        }
    }
}
