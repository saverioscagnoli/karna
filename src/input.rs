use std::{cell::OnceCell, collections::HashSet};

pub use sdl2::keyboard::Keycode as Key;
pub use sdl2::mouse::MouseButton as Mouse;

use crate::math::Vector2;

pub(crate) static mut KEYS: OnceCell<HashSet<Key>> = OnceCell::new();
pub(crate) static mut KEYS_SINGLE: OnceCell<HashSet<Key>> = OnceCell::new();
pub(crate) static mut KEYS_SINGLE_WITH_REPEAT: OnceCell<HashSet<Key>> = OnceCell::new();

pub(crate) static mut MOUSE_POSITION: OnceCell<Vector2> = OnceCell::new();
pub(crate) static mut MOUSE_BUTTONS: OnceCell<HashSet<Mouse>> = OnceCell::new();
pub(crate) static mut MOUSE_BUTTONS_SINGLE: OnceCell<HashSet<Mouse>> = OnceCell::new();

pub(crate) fn init() {
    unsafe {
        KEYS.set(HashSet::new()).unwrap();
        KEYS_SINGLE.set(HashSet::new()).unwrap();
        KEYS_SINGLE_WITH_REPEAT.set(HashSet::new()).unwrap();
        MOUSE_POSITION.set(Vector2::zero()).unwrap();
        MOUSE_BUTTONS.set(HashSet::new()).unwrap();
        MOUSE_BUTTONS_SINGLE.set(HashSet::new()).unwrap();
    }
}

pub fn key_down(key: Key) -> bool {
    unsafe { KEYS.get().unwrap().contains(&key) }
}

pub fn key_pressed(key: Key) -> bool {
    unsafe { KEYS_SINGLE.get_mut().unwrap().remove(&key) }
}

pub fn mouse_position() -> Vector2 {
    unsafe { *MOUSE_POSITION.get().unwrap() }
}

pub fn mouse_down(button: Mouse) -> bool {
    unsafe { MOUSE_BUTTONS.get().unwrap().contains(&button) }
}

pub fn click(button: Mouse) -> bool {
    unsafe { MOUSE_BUTTONS_SINGLE.get_mut().unwrap().remove(&button) }
}
