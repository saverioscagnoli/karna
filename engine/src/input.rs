use std::collections::HashMap;

use math::Vec2;
// Re-exports
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

pub(crate) struct KeyState {
    pub held: bool,
    pub pressed: bool,
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            held: false,
            pressed: false,
        }
    }
}

pub struct Input {
    pub(crate) keys: HashMap<KeyCode, KeyState>,
    pub(crate) mouse: HashMap<MouseButton, KeyState>,
    pub(crate) mouse_pos: Vec2,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: HashMap::new(),
            mouse: HashMap::new(),
            mouse_pos: Vec2::zero(),
        }
    }

    pub fn key_held(&self, key: KeyCode) -> bool {
        self.keys.get(&key).map(|s| s.held).unwrap_or(false)
    }

    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys.get(&key).map(|s| s.pressed).unwrap_or(false)
    }

    pub fn mouse_held(&self, button: MouseButton) -> bool {
        self.mouse.get(&button).map(|s| s.held).unwrap_or(false)
    }

    pub fn mouse_clicked(&self, button: MouseButton) -> bool {
        self.mouse.get(&button).map(|s| s.pressed).unwrap_or(false)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_pos
    }

    pub(crate) fn flush(&mut self) {
        self.keys.retain(|_, state| {
            state.pressed = false;
            true // Keep all entries
        });

        self.mouse.retain(|_, state| {
            state.pressed = false;
            true // Keep all entries
        });
    }
}
