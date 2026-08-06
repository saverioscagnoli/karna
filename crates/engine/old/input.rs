use std::ops::Index;
use std::ops::IndexMut;

use sdl3::gamepad::Gamepad;
use sdl3::joystick::JoystickId;
use sdl3::sys::gamepad::SDL_GamepadButton;
use sdl3::sys::init::SDL_InitFlags;
use utils::BitSet;
use utils::FastHashMap;

pub use sdl3::keyboard::Scancode as Key;
pub use sdl3::mouse::MouseButton;

use crate::window::WindowId;

#[derive(Default)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct KeySet([u64; 8]);

impl Index<usize> for KeySet {
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for KeySet {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl KeySet {
    fn split(k: Key) -> (usize, u64) {
        let i = (k as usize).min(511);
        (i / 64, 1u64 << (i % 64))
    }
}

impl BitSet for KeySet {
    type Item = Key;

    fn insert(&mut self, k: Key) {
        let (w, b) = Self::split(k);
        self[w] |= b;
    }

    fn remove(&mut self, k: Key) {
        let (w, b) = Self::split(k);
        self[w] &= !b;
    }

    fn contains(&self, k: Key) -> bool {
        let (w, b) = Self::split(k);
        self[w] & b != 0
    }

    fn clear(&mut self) {
        self.0 = [0; 8];
    }

    fn is_empty(&self) -> bool {
        self.0.iter().all(|&w| w == 0)
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MouseSet(u8);

fn mbit(btn: MouseButton) -> u8 {
    match btn {
        MouseButton::Unknown => 0,
        MouseButton::Left => 1,
        MouseButton::Right => 2,
        MouseButton::Middle => 4,
        MouseButton::X1 => 8,
        MouseButton::X2 => 16,
    }
}

impl BitSet for MouseSet {
    type Item = MouseButton;

    fn insert(&mut self, btn: MouseButton) {
        self.0 |= mbit(btn);
    }

    fn remove(&mut self, btn: MouseButton) {
        self.0 &= !mbit(btn);
    }

    fn contains(&self, btn: MouseButton) -> bool {
        let m = mbit(btn);
        m != 0 && self.0 & m != 0
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PadSet(u32);

#[derive(Clone, Copy)]
pub struct Edges<S> {
    down: S,
    press_frame: S,
    press_tick: S,
    rel_frame: S,
    rel_tick: S,
}

impl<S: Default> Default for Edges<S> {
    fn default() -> Self {
        Self {
            down: S::default(),
            press_frame: S::default(),
            press_tick: S::default(),
            rel_frame: S::default(),
            rel_tick: S::default(),
        }
    }
}

impl<S: BitSet> Edges<S> {
    pub fn press(&mut self, t: S::Item) {
        self.down.insert(t);
        self.press_frame.insert(t);
        self.press_tick.insert(t);
    }

    pub fn release(&mut self, t: S::Item) {
        self.down.remove(t);
        self.rel_frame.insert(t);
        self.rel_tick.insert(t);
    }

    pub fn held(&self, t: S::Item) -> bool {
        self.down.contains(t)
    }

    pub fn just_pressed_frame(&self, t: S::Item) -> bool {
        self.press_frame.contains(t)
    }

    pub fn just_pressed_tick(&self, t: S::Item) -> bool {
        self.press_tick.contains(t)
    }

    pub fn just_released_frame(&self, t: S::Item) -> bool {
        self.rel_frame.contains(t)
    }

    pub fn just_released_tick(&self, t: S::Item) -> bool {
        self.rel_tick.contains(t)
    }

    pub fn roll_frame(&mut self) {
        self.press_frame.clear();
        self.rel_frame.clear();
    }

    pub fn roll_tick(&mut self) {
        self.press_tick.clear();
        self.rel_tick.clear();
    }

    pub fn clear_all(&mut self) {
        self.down.clear();
        self.press_frame.clear();
        self.press_tick.clear();
        self.rel_frame.clear();
        self.rel_tick.clear();
    }
}

pub struct PadState {
    pad: Gamepad,
}

#[derive(Default)]
pub struct Input {
    pub focused: Option<WindowId>,
    pub gamepads: FastHashMap<JoystickId, Gamepad>,
    pub keys: Edges<KeySet>,
    pub mouse: Edges<MouseSet>,
    pub m_wheel: math::Vector2<f32>,
}

impl Input {
    pub fn roll_frame(&mut self) {
        self.keys.roll_frame();
        self.mouse.roll_frame();
    }

    pub fn roll_tick(&mut self) {
        self.keys.roll_tick();
        self.mouse.roll_tick();
        self.m_wheel.set([0.0, 0.0]);
    }
}
