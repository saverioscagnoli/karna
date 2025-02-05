use crate::color::Color;
use karna_opengl::{clear, clear_color, Mask};

pub struct Renderer {
    draw_color: Color,
}

impl Renderer {
    /// Marked with _ because pub(workspace) doesnt exist. :(
    /// DO NOT USE, this is for internal use only.
    pub fn _new() -> Self {
        Self {
            draw_color: Color::WHITE,
        }
    }

    /// Returns the color that will be used when drawing subsequent shapes.
    #[inline]
    pub fn color(&self) -> Color {
        self.draw_color
    }

    /// Sets the color to be used when drawing subsequent shapes.
    #[inline]
    pub fn set_color(&mut self, color: Color) {
        self.draw_color = color;
    }

    /// Sets the background color of the window.
    #[inline]
    pub fn clear_background(&self, color: Color) {
        clear_color(color.r, color.g, color.b, color.a);
    }

    /// Marked with _ because pub(workspace) doesnt exist. :(
    /// DO NOT USE, this is for internal use only.
    #[inline]
    pub fn _present(&self) {
        clear(Mask::ColorBufferBit | Mask::DepthBufferBit);
    }
}
