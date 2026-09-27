use crate::assets::AssetServer;
use crate::event::AppOutboxes;
use crate::input::Input;
use crate::render::SceneData;
use crate::render::SceneHandle;
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
        scene: &'a mut SceneData,
    ) -> LoadContext<'a> {
        #[rustfmt::skip]
        let AppOutboxes { time, window, scene: scene_outbox } = outboxes;
        let window_id = self.window_data.id();

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
            scene: SceneHandle {
                data: scene,
                outbox: scene_outbox,
            },
            shared,
        }
    }

    pub fn for_update<'a>(
        &'a mut self,
        outboxes: &'a mut AppOutboxes,
        input: &'a Input,
        assets: &'a mut AssetServer,
        shared: &'a mut SharedStore,
        scene: &'a mut SceneData,
    ) -> UpdateContext<'a> {
        #[rustfmt::skip]
        let AppOutboxes { window, time, scene: scene_outbox } = outboxes;
        let window_id = self.window_data.id();

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
            scene: SceneHandle {
                data: scene,
                outbox: scene_outbox,
            },
            shared,
        }
    }

    pub fn for_draw<'a>(
        &'a mut self,
        input: &'a Input,
        assets: &'a AssetServer,
        shared: &'a SharedStore,
        scene: &'a SceneData,
    ) -> DrawContext<'a> {
        DrawContext {
            window: &self.window_data,
            time: &self.time_data,
            input,
            assets,
            scene,
            shared,
        }
    }
}

pub struct LoadContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
    pub input: &'a Input,
    pub assets: &'a mut AssetServer,
    pub scene: SceneHandle<'a>,
    pub shared: &'a mut SharedStore,
}

pub struct UpdateContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
    pub input: &'a Input,
    pub assets: &'a mut AssetServer,
    pub scene: SceneHandle<'a>,
    pub shared: &'a mut SharedStore,
}

pub struct DrawContext<'a> {
    pub window: &'a WindowData,
    pub time: &'a TimeData,
    pub input: &'a Input,
    pub assets: &'a AssetServer,
    pub scene: &'a SceneData,
    pub shared: &'a SharedStore,
}
