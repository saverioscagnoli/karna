use std::ops::{Deref, DerefMut};

use crate::window::context::WindowContext;

pub struct WindowState {
    pub context: WindowContext,
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
