use crate::window::time::Time;

pub struct UserContext {}

pub struct LoadContext<'a> {
    pub time: &'a Time,
}

pub struct UpdateContext<'a> {
    pub time: &'a Time,
}

pub struct DrawContext<'a> {
    pub time: &'a Time,
}

impl UserContext {
    pub fn for_load<'a>(&'a mut self, time: &'a Time) -> LoadContext<'a> {
        LoadContext { time }
    }

    pub fn for_update<'a>(&'a mut self, time: &'a Time) -> UpdateContext<'a> {
        UpdateContext { time }
    }

    pub fn for_draw<'a>(&'a mut self, time: &'a Time) -> DrawContext<'a> {
        DrawContext { time }
    }
}
