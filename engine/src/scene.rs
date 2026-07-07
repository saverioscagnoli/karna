use renderer::Draw;
use utils::IndexMap;

use crate::context::ContextRef;
use crate::context::ContextRefMut;

pub type SceneBuilder = Box<dyn FnOnce(ContextRefMut) -> Box<dyn Scene> + Send>;

#[allow(unused)]
pub trait Scene: Send {
    fn load(&mut self, ctx: ContextRefMut);
    fn update(&mut self, ctx: ContextRefMut);
    fn fixed_update(&mut self, ctx: ContextRefMut) {}
    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw);
}

/// Holds the not-yet-constructed scene builders and the scenes that
/// have already been built.
#[derive(Default)]
pub struct Scenes {
    builders: IndexMap<SceneBuilder>,
    built: IndexMap<Box<dyn Scene>>,
}

impl Scenes {
    pub fn insert_builder(&mut self, label: impl Into<String>, builder: SceneBuilder) {
        self.builders.insert(label.into(), builder);
    }

    /// Builds the scene if it hasn't been built yet, running its
    /// construction closure with the given context.
    /// Returns `true` if a build actually happened.
    pub fn build(&mut self, label: &str, ctx: ContextRefMut) -> bool {
        if self.built.get(label).is_some() {
            return false;
        }

        if let Some(builder) = self.builders.remove(label) {
            let scene = builder(ctx);
            self.built.insert(label.to_string(), scene);
            true
        } else {
            false
        }
    }

    pub fn get_mut(&mut self, label: &str) -> Option<&mut Box<dyn Scene>> {
        self.built.get_mut(label)
    }
}

pub struct SceneManager {
    pub(crate) pending_activate: Vec<String>,
    pub(crate) pending_deactivate: Vec<String>,
}

impl SceneManager {
    pub(crate) fn new() -> Self {
        Self {
            pending_activate: Vec::new(),
            pending_deactivate: Vec::new(),
        }
    }

    pub fn activate<L: Into<String>>(&mut self, label: L) {
        self.pending_activate.push(label.into());
    }

    pub fn deactivate<L: Into<String>>(&mut self, label: L) {
        self.pending_deactivate.push(label.into());
    }
}
