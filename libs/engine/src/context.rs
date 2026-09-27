use crate::assets::AssetServer;
use crate::input::Input;
use crate::monitors::Monitors;
use crate::scene::SceneData;
use crate::scene::SceneHandle;
use crate::services::Services;
use crate::storage::SharedStore;
use crate::time::Time;
use crate::time::TimeData;
use crate::window::Window;
use crate::window::WindowData;

pub struct LoadContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
    pub input: &'a Input,
    pub assets: &'a mut AssetServer,
    pub scene: SceneHandle<'a>,
    pub shared: &'a mut SharedStore,
    pub monitors: &'a Monitors,
}

pub struct UpdateContext<'a> {
    pub window: Window<'a>,
    pub time: Time<'a>,
    pub input: &'a Input,
    pub assets: &'a mut AssetServer,
    pub scene: SceneHandle<'a>,
    pub shared: &'a mut SharedStore,
    pub monitors: &'a Monitors,
}

pub struct DrawContext<'a> {
    pub window: &'a WindowData,
    pub time: &'a TimeData,
    pub input: &'a Input,
    pub assets: &'a AssetServer,
    pub scene: &'a SceneData,
    pub shared: &'a SharedStore,
    pub monitors: &'a Monitors,
}

impl<'a> LoadContext<'a> {
    pub(crate) fn new(
        window_data: &'a mut WindowData,
        time_data: &'a mut TimeData,
        services: &'a mut Services,
        scene: &'a mut SceneData,
    ) -> Self {
        let Services {
            outboxes,
            input,
            assets,
            store,
            monitors,
        } = services;
        let window_id = window_data.id();

        Self {
            window: Window {
                data: window_data,
                outbox: &mut outboxes.window,
            },
            time: Time {
                window_id,
                data: time_data,
                outbox: &mut outboxes.time,
            },
            input,
            assets,
            scene: SceneHandle {
                window_id,
                data: scene,
                outbox: &mut outboxes.scene,
            },
            shared: store,
            monitors,
        }
    }
}

impl<'a> UpdateContext<'a> {
    pub(crate) fn new(
        window_data: &'a mut WindowData,
        time_data: &'a mut TimeData,
        services: &'a mut Services,
        scene: &'a mut SceneData,
    ) -> Self {
        let Services {
            outboxes,
            input,
            assets,
            store,
            monitors,
        } = services;
        let window_id = window_data.id();

        Self {
            window: Window {
                data: window_data,
                outbox: &mut outboxes.window,
            },
            time: Time {
                window_id,
                data: time_data,
                outbox: &mut outboxes.time,
            },
            input,
            assets,
            scene: SceneHandle {
                window_id,
                data: scene,
                outbox: &mut outboxes.scene,
            },
            shared: store,
            monitors,
        }
    }
}

impl<'a> DrawContext<'a> {
    pub(crate) fn new(
        window: &'a WindowData,
        time: &'a TimeData,
        services: &'a Services,
        scene: &'a SceneData,
    ) -> Self {
        Self {
            window,
            time,
            input: &services.input,
            assets: &services.assets,
            scene,
            shared: &services.store,
            monitors: &services.monitors,
        }
    }
}
