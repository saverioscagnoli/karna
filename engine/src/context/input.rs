use macros::Get;
use math::Vector2;
use wgpu::naga::FastHashSet;

// === RE-EXPORTS ===
pub use winit::{event::MouseButton, keyboard::KeyCode};

#[derive(Debug)]
#[derive(Get)]
pub struct Input {
    pub(crate) held_keys: FastHashSet<KeyCode>,
    pub(crate) pressed_keys: FastHashSet<KeyCode>,
    pub(crate) released_keys: FastHashSet<KeyCode>,

    #[get]
    pub(crate) mouse_position: Vector2,

    #[get]
    pub(crate) mouse_delta: Vector2,
    #[get(copied)]
    pub(crate) wheel_delta: f32,

    pub(crate) held_mouse: FastHashSet<MouseButton>,
    pub(crate) pressed_mouse: FastHashSet<MouseButton>,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            held_keys: FastHashSet::default(),
            pressed_keys: FastHashSet::default(),
            released_keys: FastHashSet::default(),
            mouse_position: Vector2::zeros(),
            mouse_delta: Vector2::zeros(),
            wheel_delta: 0.0,
            held_mouse: FastHashSet::default(),
            pressed_mouse: FastHashSet::default(),
        }
    }
}

impl Input {
    #[inline]
    pub fn key_held(&self, key: &KeyCode) -> bool {
        self.held_keys.contains(key)
    }

    #[inline]
    pub fn key_released(&self, key: &KeyCode) -> bool {
        self.released_keys.contains(key)
    }

    #[inline]
    pub fn key_pressed(&self, key: &KeyCode) -> bool {
        self.pressed_keys.contains(key)
    }

    #[inline]
    pub fn mouse_held(&self, button: &MouseButton) -> bool {
        self.held_mouse.contains(button)
    }

    #[inline]
    pub fn mouse_pressed(&self, button: &MouseButton) -> bool {
        self.pressed_mouse.contains(button)
    }

    #[inline]
    pub(crate) fn flush(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
        self.pressed_mouse.clear();
        self.mouse_delta.set(0.0, 0.0);
        self.wheel_delta = 0.0;
    }
}
