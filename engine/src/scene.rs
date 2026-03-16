use renderer::Draw;
use utils::FastHashMap;

use crate::ContextRef;
use crate::ContextRefMut;

pub trait Scene: Send {
    fn load(&mut self, ctx: ContextRefMut);
    fn update(&mut self, ctx: ContextRefMut);
    fn fixed_update(&mut self, ctx: ContextRefMut) {}
    fn draw(&self, ctx: ContextRef, draw: &mut Draw);
}

pub type SceneMap = FastHashMap<String, Box<dyn Scene>>;

pub struct SceneManager {
    scenes: SceneMap,
    active_scene: String,
}

impl SceneManager {
    pub(crate) fn new(scenes: SceneMap) -> Self {
        Self {
            scenes,
            active_scene: String::from("initial"),
        }
    }

    #[inline]
    pub(crate) fn load(&mut self, ctx: ContextRefMut) {
        if let Some(scene) = self.scenes.get_mut(&self.active_scene) {
            scene.load(ctx);
        }
    }

    #[inline]
    pub(crate) fn update(&mut self, ctx: ContextRefMut) {
        if let Some(scene) = self.scenes.get_mut(&self.active_scene) {
            scene.update(ctx);
        }
    }

    #[inline]
    pub(crate) fn fixed_update(&mut self, ctx: ContextRefMut) {
        if let Some(scene) = self.scenes.get_mut(&self.active_scene) {
            scene.fixed_update(ctx);
        }
    }

    #[inline]
    pub(crate) fn draw(&self, ctx: ContextRef, draw: &mut Draw) {
        if let Some(scene) = self.scenes.get(&self.active_scene) {
            scene.draw(ctx, draw);
        }
    }

    pub fn add_scene<S: Scene + 'static>(&mut self, label: impl Into<String>, scene: S) {
        self.scenes.insert(label.into(), Box::new(scene));
    }
}
