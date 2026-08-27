use crate::Input;
use crate::assets::AssetServer;
use crate::window::handle::WindowHandle;
use crate::window::time::Time;

pub struct UserContext {
    pub window: WindowHandle,
    pub time: Time,
}

pub struct ForContextMut<'app> {
    pub input: &'app Input,
    pub assets: &'app mut AssetServer,
}

pub struct ForContext<'app> {
    pub input: &'app Input,
    pub assets: &'app AssetServer,
}

pub struct LoadContext<'uctx> {
    pub window: &'uctx mut WindowHandle,
    pub time: &'uctx mut Time,
    pub input: &'uctx Input,
    pub assets: &'uctx mut AssetServer,
}

pub struct UpdateContext<'uctx> {
    pub window: &'uctx mut WindowHandle,
    pub time: &'uctx mut Time,
    pub input: &'uctx Input,
    pub assets: &'uctx mut AssetServer,
}

pub struct DrawContext<'uctx> {
    pub window: &'uctx WindowHandle,
    pub time: &'uctx Time,
    pub input: &'uctx Input,
    pub assets: &'uctx AssetServer,
}

impl UserContext {
    pub fn for_load<'a>(&'a mut self, fctx: &'a mut ForContextMut<'_>) -> LoadContext<'a> {
        LoadContext {
            window: &mut self.window,
            time: &mut self.time,
            input: fctx.input,
            assets: fctx.assets,
        }
    }

    pub fn for_update<'a>(&'a mut self, fctx: &'a mut ForContextMut<'_>) -> UpdateContext<'a> {
        UpdateContext {
            window: &mut self.window,
            time: &mut self.time,
            input: fctx.input,
            assets: fctx.assets,
        }
    }

    pub fn for_draw<'a>(&'a mut self, fctx: &ForContext<'a>) -> DrawContext<'a> {
        DrawContext {
            window: &self.window,
            time: &self.time,
            input: fctx.input,
            assets: fctx.assets,
        }
    }
}
