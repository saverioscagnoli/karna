pub mod edges;
pub mod keys;
pub mod mouse;

use crate::input::edges::Edges;
use crate::input::keys::KeySet;
use crate::input::mouse::MouseSet;
use crate::window::WindowId;

#[derive(Default)]
pub struct Input {
    pub focused: Option<WindowId>,
    pub keys: Edges<KeySet>,
    pub mouse: Edges<MouseSet>,
    pub m_wheel: math::Vector2<f32>,
}

impl Input {
    pub fn roll_tick(&mut self) {
        self.keys.roll_tick();
        self.mouse.roll_tick();
    }

    pub fn roll_frame(&mut self) {
        self.keys.roll_frame();
        self.mouse.roll_frame();
        self.m_wheel.set([0.0, 0.0]);
    }
}
