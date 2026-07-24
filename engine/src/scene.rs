use utils::FastHashMap;

use crate::window::context::ContextRef;

pub trait Scene {
    fn load(&mut self, ctx: ContextRef);
    fn update(&mut self, ctx: ContextRef);
    fn draw(&mut self, ctx: ContextRef);
}

#[derive(Default)]
pub struct SceneRegistry {
    scenes: Vec<Box<dyn Scene>>,
}

impl SceneRegistry {}
