use sdl3::event::Event;
pub use sdl3::keyboard::Keycode as Key;
pub use sdl3::keyboard::Scancode as Scan;

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

pub struct Input {
    keys: [KeyState; MAX_KEYS],
}

impl Input {
    pub(crate) fn new() -> Self {
        Self {
            keys: [KeyState::default(); MAX_KEYS],
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

    #[inline]
    pub(crate) fn flush(&mut self) {
        for key in self.keys.iter_mut() {
            key.pressed = false;
        }
    }
}
