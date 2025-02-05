use karna_math::vector::Vec2;
use sdl3::event::Event;
pub use sdl3::keyboard::Keycode as Key;
pub use sdl3::keyboard::Scancode as Scan;
pub use sdl3::mouse::MouseButton as Mouse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KeyState {
    held: bool,
    pressed: bool,
}

impl Default for KeyState {
    fn default() -> Self {
        Self {
            held: false,
            pressed: false,
        }
    }
}

const MAX_KEYS: usize = 256;
const MAX_MOUSE: usize = 5;

pub struct Input {
    keys: [KeyState; MAX_KEYS],

    mouse_position: Vec2,
    mouse: [KeyState; MAX_MOUSE],
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: [KeyState::default(); MAX_KEYS],

            mouse_position: Vec2::zero(),
            mouse: [KeyState::default(); MAX_MOUSE],
        }
    }

    #[inline]
    pub(crate) fn update(&mut self, event: &Event) {
        match event {
            Event::KeyDown {
                keycode: Some(key),
                repeat,
                ..
            } => {
                let key = *key;

                if !repeat {
                    self.keys[key as usize].pressed = true;
                }

                self.keys[key as usize].held = true;
            }

            Event::KeyUp {
                keycode: Some(key), ..
            } => {
                self.keys[*key as usize].held = false;
            }

            // Mouse events
            Event::MouseMotion { x, y, .. } => {
                self.mouse_position.x = *x as f32;
                self.mouse_position.y = *y as f32;
            }

            Event::MouseButtonDown { mouse_btn, .. } => {
                self.mouse[*mouse_btn as usize].held = true;
                self.mouse[*mouse_btn as usize].pressed = true;
            }

            Event::MouseButtonUp { mouse_btn, .. } => {
                self.mouse[*mouse_btn as usize].held = false;
            }

            _ => {}
        }
    }

    /// Checks if a key is currently being held down.
    #[inline]
    pub fn key_down(&self, key: Key) -> bool {
        self.keys[key as usize].held
    }

    /// Check if a key was pressed in the current frame.
    ///
    /// Useful for detecting single presses, such as toggleable actions.
    #[inline]
    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys[key as usize].pressed
    }

    /// Get the current mouse position.
    /// The position is relative to the window.
    #[inline]
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Checks if a mouse button is currently being held down.
    #[inline]
    pub fn mouse_down(&self, button: Mouse) -> bool {
        self.mouse[button as usize].held
    }

    /// Check if a mouse button was clicked in the current frame.
    ///
    /// Useful for detecting single clicks, such as toggleable actions.
    #[inline]
    pub fn mouse_clicked(&self, button: Mouse) -> bool {
        self.mouse[button as usize].pressed
    }

    #[inline]
    pub(crate) fn flush(&mut self) {
        for key in self.keys.iter_mut() {
            key.pressed = false;
        }

        for mouse in self.mouse.iter_mut() {
            mouse.pressed = false;
        }
    }
}
