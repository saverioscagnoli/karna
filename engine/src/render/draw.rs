use crate::render::Renderer;
use crate::render::color::Color;
use crate::window::context::FramePacket;

pub struct Draw<'ctx> {
    pub(crate) r: &'ctx mut Renderer,
    pub(crate) packet: &'ctx mut FramePacket,
}

impl<'ctx> Draw<'ctx> {
    pub fn clear_color(&self) -> Color {
        self.packet.clear_color.into()
    }

    pub fn clear_color_mut(&mut self) -> &mut math::Vector4<f32> {
        &mut self.packet.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<math::Vector4<f32>>,
    {
        self.packet.clear_color = color.into()
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {}
}
