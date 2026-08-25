use crate::assets::AssetServer;
use crate::input::Input;
use crate::window::WindowHandle;
use crate::window::time::Time;

pub struct UserContext {
    pub window_handle: WindowHandle,
}

pub struct LoadContext<'a> {
    pub window: &'a mut WindowHandle,
    pub time: &'a Time,
    pub input: &'a Input,
    pub assets: &'a mut AssetServer,
}

pub struct UpdateContext<'a> {
    pub window: &'a mut WindowHandle,
    pub time: &'a Time,
    pub input: &'a Input,
    pub assets: &'a mut AssetServer,
}

pub struct DrawContext<'a> {
    pub window: &'a WindowHandle,
    pub time: &'a Time,
    pub input: &'a Input,
    pub assets: &'a AssetServer,
}

impl UserContext {
    pub fn for_load<'a>(
        &'a mut self,
        time: &'a Time,
        input: &'a Input,
        assets: &'a mut AssetServer,
    ) -> LoadContext<'a> {
        LoadContext {
            window: &mut self.window_handle,
            time,
            input,
            assets,
        }
    }

    pub fn for_update<'a>(
        &'a mut self,
        time: &'a Time,
        input: &'a Input,
        assets: &'a mut AssetServer,
    ) -> UpdateContext<'a> {
        UpdateContext {
            window: &mut self.window_handle,
            time,
            input,
            assets,
        }
    }

    pub fn for_draw<'a>(
        &'a mut self,
        time: &'a Time,
        input: &'a Input,
        assets: &'a AssetServer,
    ) -> DrawContext<'a> {
        DrawContext {
            window: &self.window_handle,
            time,
            input,
            assets,
        }
    }
}
