use sdl3::gamepad::Gamepad;
use sdl3::gamepad::GamepadAxis;
use sdl3::gamepad::GamepadButton;
use sdl3::gamepad::GamepadId;
use sdl3::gamepad::GamepadType;
use utils::BitSet;

use crate::input::Edges;
use crate::input::InputScope;

pub const MAX_PLAYERS: usize = 4;
pub const STICK_DEADZONE: f32 = 0.2;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PadSet(u32);

impl PadSet {
    fn mask(button: GamepadButton) -> u32 {
        1u32 << (button.raw() as u32).min(31)
    }
}

impl BitSet for PadSet {
    type Item = GamepadButton;

    fn insert(&mut self, button: GamepadButton) {
        self.0 |= Self::mask(button);
    }

    fn remove(&mut self, button: GamepadButton) {
        self.0 &= !Self::mask(button);
    }

    fn contains(&self, button: GamepadButton) -> bool {
        self.0 & Self::mask(button) != 0
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stick {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    Left,
    Right,
}

pub struct Pad {
    pub(crate) device: Gamepad,
    pub(crate) buttons: Edges<PadSet>,
    pub(crate) axes: [f32; 6],
}

impl Pad {
    pub(crate) fn new(device: Gamepad) -> Self {
        Self {
            device,
            buttons: Edges::default(),
            axes: [0.0; 6],
        }
    }

    #[inline]
    pub(crate) fn id(&self) -> GamepadId {
        self.device.id()
    }

    pub(crate) fn clear(&mut self) {
        self.buttons.clear_all();
        self.axes = [0.0; 6];
    }
}

#[derive(Clone, Copy)]
pub struct PadView<'a> {
    pub(crate) pad: &'a Pad,
    pub(crate) slot: Option<usize>,
    pub(crate) scope: InputScope,
}

impl PadView<'_> {
    #[inline]
    pub fn id(&self) -> GamepadId {
        self.pad.id()
    }

    #[inline]
    pub fn slot(&self) -> Option<usize> {
        self.slot
    }

    #[inline]
    pub fn name(&self) -> &str {
        self.pad.device.name()
    }

    #[inline]
    pub fn kind(&self) -> GamepadType {
        self.pad.device.kind()
    }

    #[inline]
    pub fn down(&self, button: GamepadButton) -> bool {
        self.pad.buttons.held(button)
    }

    #[inline]
    pub fn pressed(&self, button: GamepadButton) -> bool {
        self.pad.buttons.just_pressed(button, self.scope)
    }

    #[inline]
    pub fn released(&self, button: GamepadButton) -> bool {
        self.pad.buttons.just_released(button, self.scope)
    }

    #[inline]
    pub fn axis(&self, axis: GamepadAxis) -> f32 {
        self.pad.axes[axis.raw() as usize]
    }

    pub fn stick(&self, stick: Stick) -> math::Vector2<f32> {
        let (x, y) = match stick {
            Stick::Left => (GamepadAxis::LeftX, GamepadAxis::LeftY),
            Stick::Right => (GamepadAxis::RightX, GamepadAxis::RightY),
        };

        let v = math::vec2!(self.axis(x), self.axis(y));
        let len = v.length();

        if len <= STICK_DEADZONE {
            return math::vec2!(0.0, 0.0);
        }

        let scaled = ((len - STICK_DEADZONE) / (1.0 - STICK_DEADZONE)).min(1.0);
        v * (scaled / len)
    }

    #[inline]
    pub fn left_stick(&self) -> math::Vector2<f32> {
        self.stick(Stick::Left)
    }

    #[inline]
    pub fn right_stick(&self) -> math::Vector2<f32> {
        self.stick(Stick::Right)
    }

    #[inline]
    pub fn trigger(&self, trigger: Trigger) -> f32 {
        match trigger {
            Trigger::Left => self.axis(GamepadAxis::LeftTrigger),
            Trigger::Right => self.axis(GamepadAxis::RightTrigger),
        }
    }

    #[inline]
    pub fn left_trigger(&self) -> f32 {
        self.axis(GamepadAxis::LeftTrigger)
    }

    #[inline]
    pub fn right_trigger(&self) -> f32 {
        self.axis(GamepadAxis::RightTrigger)
    }
}
