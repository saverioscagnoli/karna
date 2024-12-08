pub use sdl2::keyboard::Keycode as Key;
pub use sdl2::mouse::MouseButton as Mouse;

use crate::math::Vec2;

pub struct Input {
    pub(crate) keys: Vec<Key>,
    pub(crate) pressed_keys: Vec<Key>,

    pub(crate) mouse_buttons: Vec<Mouse>,
    pub(crate) clicked_mouse_buttons: Vec<Mouse>,
    pub(crate) mouse_position: Vec2,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: Vec::new(),
            pressed_keys: Vec::new(),
            mouse_buttons: Vec::new(),
            clicked_mouse_buttons: Vec::new(),
            mouse_position: Vec2::zero(),
        }
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.keys.contains(&key)
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub fn mouse_down(&self, button: Mouse) -> bool {
        self.mouse_buttons.contains(&button)
    }

    pub fn mouse_clicked(&self, button: Mouse) -> bool {
        self.clicked_mouse_buttons.contains(&button)
    }

    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    pub(crate) fn flush(&mut self) {
        self.pressed_keys.clear();
    }
}
