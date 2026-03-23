use utils::FastHashSet;
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

pub enum KeyState {
    Held,
    Pressed,
    Released,
}

pub struct Input {
    held_keys: FastHashSet<KeyCode>,
    pressed_keys: FastHashSet<KeyCode>,
    released_keys: FastHashSet<KeyCode>,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            held_keys: FastHashSet::default(),
            pressed_keys: FastHashSet::default(),
            released_keys: FastHashSet::default(),
        }
    }

    #[inline]
    pub(crate) fn update_keystate(&mut self, key: KeyCode, state: KeyState, remove: bool) {
        if remove {
            match state {
                KeyState::Held => self.held_keys.remove(&key),
                KeyState::Pressed => self.pressed_keys.remove(&key),
                KeyState::Released => self.pressed_keys.remove(&key),
            };
        } else {
            match state {
                KeyState::Held => self.held_keys.insert(key),
                KeyState::Pressed => self.pressed_keys.insert(key),
                KeyState::Released => self.pressed_keys.insert(key),
            };
        }
    }

    /// Returns true if a key is being held at the point of calling this function
    #[inline]
    pub fn key_held(&self, key: &KeyCode) -> bool {
        self.held_keys.contains(key)
    }

    /// Returns true if a key is pressed this frame (single-action, useful for toggles)
    #[inline]
    pub fn key_pressed(&self, key: &KeyCode) -> bool {
        self.pressed_keys.contains(key)
    }

    /// Returns true if a key was released this frame
    #[inline]
    pub fn key_released(&self, key: &KeyCode) -> bool {
        self.released_keys.contains(key)
    }

    #[inline]
    pub(crate) fn flush(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }
}
