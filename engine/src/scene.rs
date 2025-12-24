use crate::context::Context;
use macros::Get;
use math::Size;
use utils::{
    label,
    map::{Label, LabelMap},
};

#[allow(unused)]
pub trait Scene: Send {
    fn load(&mut self, ctx: &mut Context);
    fn update(&mut self, ctx: &mut Context);
    fn render(&mut self, ctx: &mut Context);

    // Optional methods

    /// Fixed update method called at a fixed rate.
    /// See [`Context::time::Time`]
    fn fixed_update(&mut self, ctx: &mut Context) {}

    /// Called when the current scene is changed to this scene.
    fn on_changed(&mut self, ctx: &mut Context) {}

    /// Called when the current scene is changed from this scene to another scene.
    fn on_changed_from(&mut self, ctx: &mut Context) {}

    /// Called when the window is resized
    fn on_resize(&mut self, size: Size<u32>, ctx: &mut Context) {}
}

#[derive(Get)]
pub struct SceneManager {
    scenes: LabelMap<Box<dyn Scene>>,
    loaded_scenes: Vec<Label>,

    #[get(name = "current_label")]
    current: Label,
}

impl SceneManager {
    pub fn new(scenes: LabelMap<Box<dyn Scene>>) -> Self {
        Self {
            scenes,
            loaded_scenes: vec![label!("initial")],
            current: label!("initial"),
        }
    }

    #[inline]
    pub fn current(&mut self) -> &mut Box<dyn Scene> {
        self.scenes
            .get_mut(&self.current)
            .expect("There's not scene set")
    }

    #[inline]
    pub fn switch_to(&mut self, label: Label, ctx: &mut Context) {
        self.current().on_changed_from(ctx);

        self.current = label;

        if !self.loaded_scenes.contains(&label) {
            self.loaded_scenes.push(label);
            self.current().load(ctx);
        }

        self.current().on_changed(ctx);
    }

    // Optional methods
}
