use crate::Input;
use crate::assets::AssetServer;
use crate::window::handle::WindowHandle;
use crate::window::text::TextHandle;
use crate::window::time::Time;

pub struct UserContext {
    pub window: WindowHandle,
    pub time: Time,
}

pub struct ForContextMut<'app> {
    pub input: &'app Input,
    pub assets: &'app mut AssetServer,
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

    pub fn for_draw<'a>(&'a self, input: &'a Input) -> DrawContext<'a> {
        DrawContext {
            window: &self.window,
            time: &self.time,
            input,
        }
    }
}

impl LoadContext<'_> {
    pub fn text(&mut self) -> TextHandle<'_> {
        let (text, atlas) = self.assets.text_targets();

        TextHandle { text, atlas }
    }
}

impl UpdateContext<'_> {
    pub fn text(&mut self) -> TextHandle<'_> {
        let (text, atlas) = self.assets.text_targets();

        TextHandle { text, atlas }
    }
}
