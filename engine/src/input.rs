use std::collections::HashSet;

use nalgebra::Vector2;
pub use winit::keyboard::KeyCode;

pub struct Input {
    pub(crate) keys_held: HashSet<KeyCode>,
    pub(crate) keys_pressed: HashSet<KeyCode>,
    pub(crate) mouse_position: Vector2<f32>,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys_held: HashSet::new(),
            keys_pressed: HashSet::new(),
            mouse_position: Vector2::default(),
        }
    }

    pub fn key_held(&self, key: &KeyCode) -> bool {
        self.keys_held.contains(key)
    }

    pub fn key_pressed(&self, key: &KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn mouse_position(&self) -> Vector2<f32> {
        self.mouse_position
    }

    pub(crate) fn flush(&mut self) {
        self.keys_pressed.clear();
    }
}
