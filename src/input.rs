use atomic_float::AtomicF32;
pub use sdl2::keyboard::Keycode as Key;
use std::{
    collections::HashSet,
    sync::{
        atomic::{self, AtomicI32},
        Arc, Mutex, MutexGuard, OnceLock,
    },
};

use crate::math::Vector2;

static KEYS: OnceLock<Arc<Mutex<HashSet<Key>>>> = OnceLock::new();
static KEYS_PRESSED: OnceLock<Arc<Mutex<HashSet<Key>>>> = OnceLock::new();
static KEYS_PRESSED_WITH_REPEAT: OnceLock<Arc<Mutex<HashSet<Key>>>> = OnceLock::new();

static MOUSE_POSITION: (AtomicI32, AtomicI32) = (AtomicI32::new(0), AtomicI32::new(0));

pub(crate) fn init() {
    KEYS.set(Arc::new(Mutex::new(HashSet::new()))).unwrap();
    KEYS_PRESSED
        .set(Arc::new(Mutex::new(HashSet::new())))
        .unwrap();
}

pub(crate) fn keys() -> MutexGuard<'static, HashSet<Key>> {
    KEYS.get().unwrap().lock().unwrap()
}

pub(crate) fn keys_pressed() -> MutexGuard<'static, HashSet<Key>> {
    KEYS_PRESSED.get().unwrap().lock().unwrap()
}

pub(crate) fn keys_pressed_with_repeat() -> MutexGuard<'static, HashSet<Key>> {
    KEYS_PRESSED_WITH_REPEAT.get().unwrap().lock().unwrap()
}

pub(crate) fn set_mouse_position(x: i32, y: i32) {
    MOUSE_POSITION.0.store(x, atomic::Ordering::Relaxed);
    MOUSE_POSITION.1.store(y, atomic::Ordering::Relaxed);
}

pub fn key_down(key: Key) -> bool {
    keys().contains(&key)
}

pub fn key_press(key: Key) -> bool {
    keys_pressed().remove(&key)
}

pub fn key_press_with_repeat(key: Key) -> bool {
    keys_pressed_with_repeat().remove(&key)
}

pub fn mouse_position() -> Vector2 {
    (
        MOUSE_POSITION.0.load(atomic::Ordering::Relaxed),
        MOUSE_POSITION.1.load(atomic::Ordering::Relaxed),
    )
        .into()
}
