use crate::conf::config;
use crate::render::color::Color;

pub struct DrawPacket {}

pub struct Draw {
    color: Color,
}

impl Draw {
    pub fn new() -> Self {
        let conf = config();

        Self {
            color: conf.draw_color,
        }
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn color_mut(&mut self) -> &mut Color {
        &mut self.color
    }

    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.color = color.into();
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {}
}
