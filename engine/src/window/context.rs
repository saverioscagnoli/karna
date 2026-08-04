use crate::render::scene_ref::SceneRef;
use crate::window::WindowHandle;

pub struct UserContext {
    pub window: WindowHandle,
}

pub struct LoadContext<'a> {
    window: &'a mut WindowHandle,
}

pub struct UpdateContext<'a> {
    window: &'a mut WindowHandle,
}

pub struct DrawContext<'a> {
    window: &'a mut WindowHandle,
}

impl UserContext {
    pub fn split_load<'a>(
        &'a mut self,
        scene: &'a mut SceneRef,
    ) -> (LoadContext<'a>, &'a mut SceneRef) {
        (
            LoadContext {
                window: &mut self.window,
            },
            scene,
        )
    }
}
