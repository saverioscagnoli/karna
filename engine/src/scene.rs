use crate::context::Context;
use std::collections::HashMap;
use traccia::error;

pub trait Scene {
    fn load(&mut self, ctx: &mut Context);
    fn fixed_update(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context);
    fn render(&mut self, ctx: &mut Context);
}

pub struct SceneManager {
    pub(crate) current: String,
    scenes: HashMap<String, Box<dyn Scene>>,
}

impl SceneManager {
    pub(crate) fn new() -> Self {
        Self {
            current: String::new(),
            scenes: HashMap::new(),
        }
    }

    pub fn add<S: AsRef<str>>(&mut self, name: S, scene: Box<dyn Scene>) {
        let name = name.as_ref().to_string();

        if self.scenes.contains_key(&name) {
            error!("Scene '{}' already exists", name);
        } else {
            self.scenes.insert(name.clone(), scene);
        }
    }

    pub fn switch_to<S: AsRef<str>>(&mut self, scene_name: S, ctx: &mut Context) {
        if let Some(scene) = self.scenes.get_mut(scene_name.as_ref()) {
            self.current = scene_name.as_ref().to_string();
            scene.load(ctx);
        } else {
            error!("Scene '{}' not found", scene_name.as_ref());
        }
    }

    pub(crate) fn current_mut(&mut self) -> Option<&mut Box<dyn Scene>> {
        self.scenes.get_mut(&self.current)
    }
}
