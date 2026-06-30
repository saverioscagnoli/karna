use math::Size;
use math::Vector2;
use math::Vector4;

use crate::Color;
use crate::Renderer;

pub struct Draw<'r> {
    renderer: &'r mut Renderer,
    color: Vector4<f32>,
}

impl<'r> Draw<'r> {
    #[doc(hidden)]
    pub fn _new(r: &'r mut Renderer) -> Self {
        Self {
            renderer: r,
            color: Color::White.into(),
        }
    }

    pub fn color(&self) -> Color {
        self.color.into()
    }

    pub fn set_color<C: Into<Vector4<f32>>>(&mut self, color: C) {
        self.color = color.into();
    }

    pub fn point(&mut self, x: f32, y: f32) {
        let layer = self.renderer.active_layer_mut();
        layer.immediate.push_point(x, y, self.color);
    }

    pub fn point_v<P: Into<Vector2<f32>>>(&mut self, pos: P) {
        let pos: Vector2<f32> = pos.into();
        self.point(pos.x, pos.y);
    }

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let layer = self.renderer.active_layer_mut();
        layer.immediate.push_quad(x, y, w, h, self.color);
    }

    pub fn rect_v<P: Into<Vector2<f32>>, S: Into<Size<f32>>>(&mut self, pos: P, size: S) {
        let pos: Vector2<f32> = pos.into();
        let size: Size<f32> = size.into();
        self.rect(pos.x, pos.y, size.width, size.height);
    }
}
