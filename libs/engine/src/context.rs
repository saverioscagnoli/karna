use crate::assets::AssetServer;
use crate::event::AppOutboxes;
use crate::input::Input;
use crate::storage::SharedStore;
use crate::time::Time;
use crate::time::TimeData;
use crate::window::Window;
use crate::window::WindowData;

pub struct UserContext {
    pub window_data: WindowData,
    pub time_data: TimeData,
}

impl UserContext {
    pub fn for_load<'a>(
        &'a mut self,
        outboxes: &'a mut AppOutboxes,
        input: &'a Input,
        assets: &'a mut AssetServer,
        shared: &'a mut SharedStore,
    ) -> LoadContext<'a> {
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
            input,
            assets,
            shared,
        }
    }

    pub fn for_update<'a>(
        &'a mut self,
        outboxes: &'a mut AppOutboxes,
        input: &'a Input,
        assets: &'a mut AssetServer,
        shared: &'a mut SharedStore,
    ) -> UpdateContext<'a> {
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
            input,
            assets,
            shared,
        }
    }

    pub fn for_draw<'a>(
        &'a mut self,
        outboxes: &'a mut AppOutboxes,
        input: &'a Input,
        assets: &'a AssetServer,
        shared: &'a SharedStore,
    ) -> DrawContext<'a> {
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
            input,
            assets,
            shared,
        }
    }
}

pub struct LoadContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
    pub input: &'a Input,
    pub assets: &'a mut AssetServer,
    pub shared: &'a mut SharedStore,
}

pub struct UpdateContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
    pub input: &'a Input,
    pub assets: &'a mut AssetServer,
    pub shared: &'a mut SharedStore,
}

pub struct DrawContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
    pub input: &'a Input,
    pub assets: &'a AssetServer,
    pub shared: &'a SharedStore,
}
