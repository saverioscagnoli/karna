use std::collections::HashMap;

// Re-exports
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
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: HashMap::new(),
        }
    }

    pub fn key_held(&self, key: KeyCode) -> bool {
        self.keys.get(&key).map(|s| s.held).unwrap_or(false)
    }

    pub fn key_pressed(&self, key: KeyCode) -> bool {
        self.keys.get(&key).map(|s| s.pressed).unwrap_or(false)
    }

    pub(crate) fn flush(&mut self) {
        for state in self.keys.values_mut() {
            state.pressed = false;
        }
    }
}
