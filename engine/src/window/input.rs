use utils::FastHashSet;

pub use sdl3::keyboard::Scancode as Key;
pub use sdl3::mouse::MouseButton;

#[derive(Default)]
pub struct Input {
    pub(crate) k_down: FastHashSet<Key>,
    pub(crate) k_pressed: FastHashSet<Key>,
    pub(crate) k_released: FastHashSet<Key>,
    pub(crate) m_down: FastHashSet<MouseButton>,
    pub(crate) m_pressed: FastHashSet<MouseButton>,
    pub(crate) m_released: FastHashSet<MouseButton>,
    pub(crate) m_pos: math::Vector2<f32>,
    pub(crate) m_delta: math::Vector2<f32>,
    pub(crate) m_wheel: math::Vector2<f32>,
}

impl Input {
    pub fn key_down(&self, key: Key) -> bool {
        self.k_down.contains(&key)
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.k_pressed.contains(&key)
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.k_released.contains(&key)
    }

    pub fn mouse_down(&self, btn: MouseButton) -> bool {
        self.m_down.contains(&btn)
    }

    pub fn mouse_pressed(&self, btn: MouseButton) -> bool {
        self.m_pressed.contains(&btn)
    }

    pub fn mouse_released(&self, btn: MouseButton) -> bool {
        self.m_released.contains(&btn)
    }

    pub fn mouse_position(&self) -> math::Vector2<f32> {
        self.m_pos
    }

    pub fn mouse_delta(&self) -> math::Vector2<f32> {
        self.m_delta
    }

    pub fn mouse_wheel(&self) -> math::Vector2<f32> {
        self.m_wheel
    }

    pub(crate) fn flush(&mut self) {
        self.k_pressed.clear();
        self.k_released.clear();
        self.m_pressed.clear();
        self.m_released.clear();
        self.m_wheel.set([0.0, 0.0]);
    }
}
