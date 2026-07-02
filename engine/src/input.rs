pub use sokol::app::Keycode;
pub use sokol::app::Mousebutton;
use utils::FastHashSet;

pub struct Input {
    // Keyboard
    pub(crate) held_keys: FastHashSet<Keycode>,
    pub(crate) pressed_keys: FastHashSet<Keycode>,
    pub(crate) released_keys: FastHashSet<Keycode>,

    // Mouse
    pub(crate) mouse_position: math::Vector2<f32>,
    pub(crate) mouse_delta: math::Vector2<f32>,
    pub(crate) wheel_delta: f32,
    pub(crate) held_mouse_buttons: FastHashSet<Mousebutton>,
    pub(crate) pressed_mouse_buttons: FastHashSet<Mousebutton>,
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            held_keys: FastHashSet::default(),
            pressed_keys: FastHashSet::default(),
            released_keys: FastHashSet::default(),
            mouse_position: math::Vector2::zero(),
            mouse_delta: math::Vector2::zero(),
            wheel_delta: 0.0,
            held_mouse_buttons: FastHashSet::default(),
            pressed_mouse_buttons: FastHashSet::default(),
        }
    }

    /// Returns true if the given key is being held down
    pub fn key_held(&self, k: &Keycode) -> bool {
        self.held_keys.contains(k)
    }

    /// Returns true if the given is pressed, but it
    /// does not persist across frame, so it can be useful
    /// for one-time actions, such as toggling
    pub fn key_pressed(&self, k: &Keycode) -> bool {
        self.pressed_keys.contains(k)
    }

    /// Returns true if the given key was released this frame
    pub fn key_released(&self, k: &Keycode) -> bool {
        self.released_keys.contains(k)
    }

    pub fn mouse_position(&self) -> &math::Vector2<f32> {
        &self.mouse_position
    }

    pub fn mouse_delta(&self) -> &math::Vector2<f32> {
        &self.mouse_delta
    }

    pub fn whele_delta(&self) -> f32 {
        self.wheel_delta
    }

    /// Returns true if the given mouse button is being held down
    pub fn mouse_held(&self, b: &Mousebutton) -> bool {
        self.held_mouse_buttons.contains(b)
    }

    /// Returns true if the given mouse button is pressed,
    /// but it does not persist across frames, so it can be useful
    /// for one-time actions, such as clicking something
    pub fn mouse_pressed(&self, b: &Mousebutton) -> bool {
        self.pressed_mouse_buttons.contains(b)
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
        self.wheel_delta = 0.0;
    }
}
