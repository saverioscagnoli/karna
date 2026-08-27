use crate::window::handle::WindowHandle;
use crate::window::time::Time;

pub struct UserContext {
    pub window: WindowHandle,
    pub time: Time,
}

pub struct LoadContext<'uctx> {
    pub window: &'uctx mut WindowHandle,
    pub time: &'uctx mut Time,
}

pub struct UpdateContext<'uctx> {
    pub window: &'uctx mut WindowHandle,
    pub time: &'uctx mut Time,
}

pub struct DrawContext<'uctx> {
    pub window: &'uctx WindowHandle,
    pub time: &'uctx Time,
}

impl UserContext {
    pub fn for_load<'a>(&'a mut self) -> LoadContext<'a> {
        LoadContext {
            window: &mut self.window,
            time: &mut self.time,
        }
    }

    pub fn for_update<'a>(&'a mut self) -> UpdateContext<'a> {
        UpdateContext {
            window: &mut self.window,
            time: &mut self.time,
        }
    }

    pub fn for_draw<'a>(&'a mut self) -> DrawContext<'a> {
        DrawContext {
            window: &self.window,
            time: &self.time,
        }
    }
}
