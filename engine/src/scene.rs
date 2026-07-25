use utils::FastHashMap;

use crate::{
    render::{immediate::Draw, retained::SceneRef},
    window::context::ContextRef,
};

pub trait Scene: Send {
    fn load(&mut self, ctx: ContextRef, scene: &mut SceneRef);
    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef);
    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw);
}

pub struct SceneRegistry {
    pub(crate) scenes: FastHashMap<String, Box<dyn Scene>>,
}

impl SceneRegistry {
    pub fn new(scenes: FastHashMap<String, Box<dyn Scene>>) -> Self {
        Self { scenes }
    }

    pub fn get_mut(&mut self, label: &str) -> &mut Box<dyn Scene> {
        self.scenes.get_mut(label).expect("Failed to get scene")
    }
}
