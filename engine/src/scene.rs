use std::any::Any;

use utils::FastHashMap;

use crate::render::Draw;
use crate::window::context::Context;

pub trait Scene: Send {
    fn load(&mut self, ctx: Context);

    fn fixed_update(&mut self, ctx: Context) {
        _ = ctx
    }

    fn update(&mut self, ctx: Context);

    fn draw(&mut self, ctx: Context, draw: &mut Draw);
}

#[derive(Default)]
pub struct SceneRegistry {
    scenes: FastHashMap<usize, Box<dyn Scene>>,
}

impl SceneRegistry {
    pub fn new() -> Self {
        Self {
            scenes: FastHashMap::default(),
        }
    }

    pub fn insert<S>(&mut self, i: usize, scene: S)
    where
        S: Scene + 'static,
    {
        self.scenes.insert(i, Box::new(scene));
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Box<dyn Scene> {
        self.scenes.get_mut(&index).expect("Failed to get scene")
    }
}

pub enum SceneManagerCommand {
    Activate(usize, Box<dyn Any>),
    Deactivate(usize),
    Pause(usize),
    Resume(usize),
}

pub struct SceneManager {
    pub(crate) buffer: Vec<SceneManagerCommand>,
}

impl SceneManager {
    pub(crate) fn new() -> Self {
        Self { buffer: Vec::new() }
    }
}
