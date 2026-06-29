use math::Vector2;
use math::Vector3;
use math::Vector4;

use crate::Color;
use crate::Renderer;
use crate::Vertex;

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

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let id = self.renderer.world;
        let layer = self.renderer.layer_mut(&id);

        layer.immediate.triangle_batcher.push_quad([
            Vertex::new(Vector3::new(x, y, 0.0), self.color, Vector2::new(0.0, 0.0)),
            Vertex::new(
                Vector3::new(x + w, y, 0.0),
                self.color,
                Vector2::new(1.0, 0.0),
            ),
            Vertex::new(
                Vector3::new(x, y + h, 0.0),
                self.color,
                Vector2::new(0.0, 1.0),
            ),
            Vertex::new(
                Vector3::new(x + w, y + h, 0.0),
                self.color,
                Vector2::new(1.0, 1.0),
            ),
        ]);
    }
}
