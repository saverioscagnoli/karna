use math::Vector2;
use utils::FastHashSet;
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode as Keycode;

pub struct Input {
    // Keyboard
    pub(crate) held_keys: FastHashSet<Keycode>,
    pub(crate) pressed_keys: FastHashSet<Keycode>,
    pub(crate) released_keys: FastHashSet<Keycode>,

    // Mouse
    pub(crate) mouse_position: Vector2<f32>,
    pub(crate) mouse_delta: Vector2<f32>,
    pub(crate) wheel_delta: Vector2<f32>,
    pub(crate) held_mouse_buttons: FastHashSet<MouseButton>,
    pub(crate) pressed_mouse_buttons: FastHashSet<MouseButton>,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            held_keys: FastHashSet::default(),
            pressed_keys: FastHashSet::default(),
            released_keys: FastHashSet::default(),
            mouse_position: Vector2::zero(),
            mouse_delta: Vector2::zero(),
            wheel_delta: Vector2::zero(),
            held_mouse_buttons: FastHashSet::default(),
            pressed_mouse_buttons: FastHashSet::default(),
        }
    }

    pub fn key_axis(&self, keys: [Keycode; 2]) -> f32 {
        if self.key_held(keys[0]) {
            return -1.0;
        }

        if self.key_held(keys[1]) {
            return 1.0;
        }

        0.0
    }

    /// Returns true if the given key is being held down
    pub fn key_held(&self, k: Keycode) -> bool {
        self.held_keys.contains(&k)
    }

    /// Returns true if the given is pressed, but it
    /// does not persist across frame, so it can be useful
    /// for one-time actions, such as toggling
    pub fn key_pressed(&self, k: Keycode) -> bool {
        self.pressed_keys.contains(&k)
    }

    /// Returns true if the given key was released this frame
    pub fn key_released(&self, k: &Keycode) -> bool {
        self.released_keys.contains(k)
    }

    pub fn mouse_position(&self) -> Vector2<f32> {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> Vector2<f32> {
        self.mouse_delta
    }

    pub fn whele_delta(&self) -> Vector2<f32> {
        self.wheel_delta
    }

    /// Returns true if the given mouse button is being held down
    pub fn mouse_held(&self, b: MouseButton) -> bool {
        self.held_mouse_buttons.contains(&b)
    }

    /// Returns true if the given mouse button is pressed,
    /// but it does not persist across frames, so it can be useful
    /// for one-time actions, such as clicking something
    pub fn mouse_pressed(&self, b: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&b)
    }

    /// Clears every input type that
    /// must not persits across frames.
    ///
    /// (i.e.) key presses, one-time
    pub fn flush(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
        self.pressed_mouse_buttons.clear();
        self.mouse_delta.set([0.0, 0.0]);
        self.wheel_delta.set([0.0, 0.0]);
    }
}
