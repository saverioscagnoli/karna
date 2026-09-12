use crate::event::AppOutboxes;
use crate::time::Time;
use crate::time::TimeData;
use crate::window::Window;
use crate::window::WindowData;

pub struct UserContext {
    pub window_data: WindowData,
    pub time_data: TimeData,
}

impl UserContext {
    pub fn for_load<'a>(&'a mut self, outboxes: &'a mut AppOutboxes) -> LoadContext<'a> {
        let window_id = self.window_data.id();
        let AppOutboxes { time, window } = outboxes;

        LoadContext {
            window: Window {
                data: &mut self.window_data,
                outbox: window,
            },
            time: Time {
                window_id,
                data: &mut self.time_data,
                outbox: time,
            },
        }
    }

    pub fn for_update<'a>(&'a mut self, outboxes: &'a mut AppOutboxes) -> UpdateContext<'a> {
        let window_id = self.window_data.id();
        let AppOutboxes { window, time } = outboxes;

        UpdateContext {
            window: Window {
                data: &mut self.window_data,
                outbox: window,
            },
            time: Time {
                window_id,
                data: &mut self.time_data,
                outbox: time,
            },
        }
    }

    pub fn for_draw<'a>(&'a mut self, outboxes: &'a mut AppOutboxes) -> DrawContext<'a> {
        let window_id = self.window_data.id();
        let AppOutboxes { window, time } = outboxes;

        DrawContext {
            window: Window {
                data: &mut self.window_data,
                outbox: window,
            },
            time: Time {
                window_id,
                data: &mut self.time_data,
                outbox: time,
            },
        }
    }
}

pub struct LoadContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
}

pub struct UpdateContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
}

pub struct DrawContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
}
