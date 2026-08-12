pub mod edges;
pub mod keys;
pub mod mouse;

use crate::input::edges::Edges;
use crate::input::edges::InputScope;
use crate::input::keys::Key;
use crate::input::keys::KeySet;
use crate::input::mouse::MouseButton;
use crate::input::mouse::MouseSet;
use crate::window::WindowId;

#[derive(Default)]
pub struct Input {
    pub(crate) focused: Option<WindowId>,
    pub(crate) scope: InputScope,
    pub(crate) keys: Edges<KeySet>,
    pub(crate) mouse: Edges<MouseSet>,
    pub(crate) m_wheel: math::Vector2<f32>,
}

impl Input {
    pub(crate) fn roll_tick(&mut self) {
        self.keys.roll_tick();
        self.mouse.roll_tick();
    }

    pub(crate) fn roll_frame(&mut self) {
        self.keys.roll_frame();
        self.mouse.roll_frame();
        self.m_wheel.set([0.0, 0.0]);
    }

    pub(crate) fn change_scope(&mut self, scope: InputScope) {
        self.scope = scope;
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
}
