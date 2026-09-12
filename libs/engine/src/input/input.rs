use nostd::alloc::string::String;
use sdl3::events::Key;
use sdl3::events::MouseButton;
use sdl3::window::WindowId;

use crate::input::Edges;
use crate::input::InputScope;
use crate::input::KeySet;
use crate::input::MouseSet;

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

    pub(crate) fn roll_tick(&mut self) {
        self.keys.roll_tick();
        self.mouse.roll_tick();
    }

    pub(crate) fn roll_frame(&mut self) {
        self.keys.roll_frame();
        self.mouse.roll_frame();
        self.m_wheel.set([0.0, 0.0]);
        self.text.clear();
    }

    pub(crate) fn change_scope(&mut self, scope: InputScope) {
        self.scope = scope;
    }
}
