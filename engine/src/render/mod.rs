use crate::render::draw::DrawPacket;
use crate::window::platform::PlatformWindow;

pub mod color;
pub mod draw;
pub mod layer;
pub mod stage;

pub struct Renderer {}

impl Renderer {
    pub fn present(&mut self, window: &PlatformWindow, packet: &DrawPacket) {}
}
