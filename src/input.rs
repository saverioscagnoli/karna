use std::collections::HashSet;

pub use sdl2::keyboard::Keycode as Key;
pub use sdl2::mouse::MouseButton as Mouse;

use crate::math::Vec2;

pub struct Input {
    pub(crate) keys: HashSet<Key>,
    pub(crate) keys_pressed: HashSet<Key>,
    pub(crate) mouse_position: Vec2,
    pub(crate) mouse_buttons: HashSet<Mouse>,
    pub(crate) mouse_buttons_pressed: HashSet<Mouse>,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: HashSet::new(),
            keys_pressed: HashSet::new(),
            mouse_position: Vec2::zero(),
            mouse_buttons: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
        }
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub fn mouse_down(&self, button: Mouse) -> bool {
        self.mouse_buttons.contains(&button)
    }

    pub fn mouse_clicked(&self, button: Mouse) -> bool {
        self.mouse_buttons_pressed.contains(&button)
    }

    pub(crate) fn flush(&mut self) {
        self.keys_pressed.clear();
        self.mouse_buttons_pressed.clear();
    }
}
