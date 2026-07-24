use std::ops::{Deref, DerefMut};

use crate::{scene::SceneRegistry, window::context::WindowContext};

pub struct WindowState {
    pub context: WindowContext,
    pub scenes: SceneRegistry,
}

impl Deref for WindowState {
    type Target = WindowContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for WindowState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}
