use nostd::alloc::string::String;
use nostd::alloc::vec::Vec;
use sdl3::events::Key;
use sdl3::events::MouseButton;
use sdl3::gamepad::Gamepad;
use sdl3::gamepad::GamepadAxis;
use sdl3::gamepad::GamepadButton;
use sdl3::gamepad::GamepadId;
use sdl3::window::WindowId;

use crate::input::Edges;
use crate::input::InputScope;
use crate::input::KeySet;
use crate::input::MAX_PLAYERS;
use crate::input::MouseSet;
use crate::input::Pad;
use crate::input::PadView;

#[derive(Default)]
pub struct Input {
    pub(crate) focused: Option<WindowId>,
    pub(crate) scope: InputScope,
    pub(crate) keys: Edges<KeySet>,
    pub(crate) mouse: Edges<MouseSet>,
    pub(crate) m_wheel: math::Vector2<f32>,
    pub(crate) text: String,
    pub(crate) preedit: String,
    pub(crate) preedit_cursor: i32,
    pub(crate) pads: Vec<Pad>,
    pub(crate) slots: [Option<GamepadId>; MAX_PLAYERS],
}

impl Input {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn preedit(&self) -> &str {
        &self.preedit
    }

    pub fn preedit_cursor(&self) -> i32 {
        self.preedit_cursor
    }

    pub fn key_down(&self, key: Key) -> bool {
        self.keys.held(key)
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.keys.just_pressed(key, self.scope)
    }

    pub fn key_released(&self, key: Key) -> bool {
        self.keys.just_released(key, self.scope)
    }

    pub fn mouse_down(&self, btn: MouseButton) -> bool {
        self.mouse.held(btn)
    }

    pub fn mouse_pressed(&self, btn: MouseButton) -> bool {
        self.mouse.just_pressed(btn, self.scope)
    }

    pub fn mouse_released(&self, btn: MouseButton) -> bool {
        self.mouse.just_released(btn, self.scope)
    }

    pub fn mouse_wheel(&self) -> math::Vector2<f32> {
        self.m_wheel
    }

    pub fn pad(&self, slot: usize) -> Option<PadView<'_>> {
        self.pad_by_id((*self.slots.get(slot)?)?)
    }

    pub fn pad_by_id(&self, id: GamepadId) -> Option<PadView<'_>> {
        self.pads
            .iter()
            .find(|p| p.id() == id)
            .map(|pad| self.view(pad))
    }

    pub fn pads(&self) -> impl Iterator<Item = PadView<'_>> {
        self.pads.iter().map(|pad| self.view(pad))
    }

    pub fn slot_connected(&self, slot: usize) -> bool {
        self.slots.get(slot).is_some_and(Option::is_some)
    }

    pub fn slot_of(&self, id: GamepadId) -> Option<usize> {
        self.slots.iter().position(|s| *s == Some(id))
    }

    pub fn any_pad_down(&self, button: GamepadButton) -> bool {
        self.pads().any(|p| p.down(button))
    }

    pub fn any_pad_pressed(&self, button: GamepadButton) -> bool {
        self.pads().any(|p| p.pressed(button))
    }

    pub fn any_pad_released(&self, button: GamepadButton) -> bool {
        self.pads().any(|p| p.released(button))
    }

    fn view<'a>(&'a self, pad: &'a Pad) -> PadView<'a> {
        PadView {
            pad,
            slot: self.slot_of(pad.id()),
            scope: self.scope,
        }
    }

    pub(crate) fn connect_pad(&mut self, device: Gamepad) {
        self.pads.push(Pad::new(device));
        self.fill_slots();
    }

    pub(crate) fn disconnect_pad(&mut self, id: GamepadId) -> Option<Pad> {
        for slot in &mut self.slots {
            if *slot == Some(id) {
                *slot = None;
            }
        }

        let index = self.pads.iter().position(|p| p.id() == id)?;
        let pad = self.pads.swap_remove(index);

        self.fill_slots();
        Some(pad)
    }

    pub(crate) fn slot_pad_mut(&mut self, slot: usize) -> Option<&mut Pad> {
        let id = (*self.slots.get(slot)?)?;
        self.pads.iter_mut().find(|p| p.id() == id)
    }

    fn fill_slots(&mut self) {
        for i in 0..self.pads.len() {
            let id = self.pads[i].id();

            if self.slot_of(id).is_some() {
                continue;
            }

            let Some(free) = self.slots.iter().position(Option::is_none) else {
                break;
            };

            self.slots[free] = Some(id);
            _ = self.pads[i].device.set_player_index(Some(free as u32));
        }
    }

    pub(crate) fn pad_button(&mut self, id: GamepadId, button: GamepadButton, pressed: bool) {
        let focused = self.focused.is_some();

        let Some(pad) = self.pads.iter_mut().find(|p| p.id() == id) else {
            return;
        };

        if pressed {
            if focused {
                pad.buttons.press(button);
            }
        } else {
            pad.buttons.release(button);
        }
    }

    pub(crate) fn pad_axis(&mut self, id: GamepadId, axis: GamepadAxis, value: f32) {
        if let Some(pad) = self.pads.iter_mut().find(|p| p.id() == id) {
            pad.axes[axis.raw() as usize] = value;
        }
    }

    pub(crate) fn clear_held(&mut self) {
        self.keys.clear_all();
        self.mouse.clear_all();

        for pad in &mut self.pads {
            pad.clear();
        }
    }

    pub(crate) fn roll_tick(&mut self) {
        self.keys.roll_tick();
        self.mouse.roll_tick();

        for pad in &mut self.pads {
            pad.buttons.roll_tick();
        }
    }

    pub(crate) fn roll_frame(&mut self) {
        self.keys.roll_frame();
        self.mouse.roll_frame();

        for pad in &mut self.pads {
            pad.buttons.roll_frame();
        }

        self.m_wheel.set([0.0, 0.0]);
        self.text.clear();
    }

    pub(crate) fn change_scope(&mut self, scope: InputScope) {
        self.scope = scope;
    }
}
