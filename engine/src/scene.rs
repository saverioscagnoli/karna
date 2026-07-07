use std::any::Any;

use renderer::Draw;
use utils::IndexMap;

use crate::context::ContextRef;
use crate::context::ContextRefMut;

pub type SceneBuilder = Box<dyn FnOnce(ContextRefMut) -> Box<dyn Scene> + Send>;

#[allow(unused)]
pub trait Scene: Send {
    fn load(&mut self, ctx: ContextRefMut);
    fn loaded_with(&mut self, ctx: ContextRefMut, user_data: Box<dyn Any>) {}
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
    pub fn insert_builder<L: Into<String>>(&mut self, label: L, builder: SceneBuilder) {
        let label: String = label.into();

        if !self.builders.contains_key(&label) {
            self.builders.insert(label.into(), builder);
        }
    }

    pub fn register<L: Into<String>>(&mut self, label: L, builder: SceneBuilder) {
        self.insert_builder(label, builder);
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

pub enum SceneCommand {
    Register {
        label: String,
        scene: Box<dyn Scene>,
    },
    Activate {
        label: String,
        user_data: Option<Box<dyn Any>>,
    },
    Deactivate {
        label: String,
    },
}

pub struct SceneManager {
    buffer: Vec<SceneCommand>,
}

impl SceneManager {
    pub(crate) fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn register<L: Into<String>>(&mut self, label: L, scene: Box<dyn Scene>) {
        self.buffer.push(SceneCommand::Register {
            label: label.into(),
            scene,
        })
    }

    pub fn activate<L: Into<String>>(&mut self, label: L) {
        self.buffer.push(SceneCommand::Activate {
            label: label.into(),
            user_data: None,
        })
    }

    pub fn activate_with<L: Into<String>, D: Any>(&mut self, label: L, user_data: D) {
        self.buffer.push(SceneCommand::Activate {
            label: label.into(),
            user_data: Some(Box::new(user_data)),
        })
    }

    pub fn deactivate<L: Into<String>>(&mut self, label: L) {
        self.buffer.push(SceneCommand::Deactivate {
            label: label.into(),
        })
    }

    pub(crate) fn drain_collect(&mut self) -> Vec<SceneCommand> {
        self.buffer.drain(..).collect()
    }
}
