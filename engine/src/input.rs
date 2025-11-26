use macros_derive::Getters;
use math::Vector2;
use std::collections::HashSet;
pub use winit::keyboard::KeyCode;

#[rustfmt::skip]
#[derive(Debug, Clone)]
#[derive(Getters)]
pub struct Input {
    pub(crate) keys_held: HashSet<KeyCode>,
    pub(crate) keys_pressed: HashSet<KeyCode>,

    #[get(copied)]
    pub(crate) mouse_position: Vector2,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys_held: HashSet::new(),
            keys_pressed: HashSet::new(),
            mouse_position: Vector2::zero(),
        }
    }

    pub fn key_held(&self, key: &KeyCode) -> bool {
        self.keys_held.contains(key)
    }

    pub fn key_pressed(&self, key: &KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub(crate) fn flush(&mut self) {
        self.keys_pressed.clear();
    }
}
