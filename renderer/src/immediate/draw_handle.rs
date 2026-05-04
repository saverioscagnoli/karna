use assets::AssetServerGuard;
use assets::Image;
use math::Size;
use math::Vector2;
use math::Vector4;
use utils::Handle;

use crate::Color;
use crate::Renderer;

pub struct Draw<'r> {
    assets: AssetServerGuard<'r>,
    renderer: &'r mut Renderer,
}

impl<'r> Draw<'r> {
    #[inline]
    #[doc(hidden)]
    pub fn new(assets: AssetServerGuard<'r>, renderer: &'r mut Renderer) -> Self {
        Self { assets, renderer }
    }

    #[inline]
    pub fn color(&self) -> Color {
        (*self.renderer.active_layer().immediate().draw_color()).into()
    }

    #[inline]
    pub fn set_color<C: Into<Vector4>>(&mut self, color: C) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .set_draw_color(color);
    }

    #[inline]
    pub fn scale(&self) -> f32 {
        self.renderer.active_layer().immediate().scale()
    }

    #[inline]
    pub fn set_scale(&mut self, scale: f32) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .set_scale(scale);
    }

    #[inline]
    pub fn reset_scale(&mut self) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .set_scale(1.0);
    }

    #[inline]
    pub fn point(&mut self, x: f32, y: f32) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .push_point(&self.assets, x, y);
    }

    #[inline]
    pub fn point_v<P: Into<Vector2>>(&mut self, pos: P) {
        let pos: Vector2 = pos.into();

        self.point(pos.x, pos.y);
    }

    #[inline]
    pub fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .push_line(&self.assets, x1, y1, x2, y2);
    }

    #[inline]
    pub fn line_v<P: Into<Vector2>, Q: Into<Vector2>>(&mut self, pos1: P, pos2: Q) {
        let pos1: Vector2 = pos1.into();
        let pos2: Vector2 = pos2.into();

        self.line(pos1.x, pos1.y, pos2.x, pos2.y);
    }

    #[inline]
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .push_quad(&self.assets, x, y, w, h);
    }

    #[inline]
    pub fn rect_v<P: Into<Vector2>, S: Into<Size<f32>>>(&mut self, pos: P, size: S) {
        let pos: Vector2 = pos.into();
        let size: Size<f32> = size.into();

        self.rect(pos.x, pos.y, size.width, size.height);
    }

    #[inline]
    pub fn circle(&mut self, x: f32, y: f32, r: f32) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .push_circle(x, y, r);
    }

    #[inline]
    pub fn circle_v<P: Into<Vector2>>(&mut self, pos: P, r: f32) {
        let pos: Vector2 = pos.into();

        self.circle(pos.x, pos.y, r);
    }

    #[inline]
    pub fn image(&mut self, image: Handle<Image>, x: f32, y: f32) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .push_textured_quad(image, &self.assets, x, y);
    }

    #[inline]
    pub fn image_v<P: Into<Vector2>>(&mut self, image: Handle<Image>, pos: P) {
        let pos: Vector2 = pos.into();

        self.image(image, pos.x, pos.y);
    }

    #[inline]
    pub fn atlas(&mut self, x: f32, y: f32) {
        self.renderer
            .active_layer_mut()
            .immediate_mut()
            .push_textured_quad(self.assets.atlas_handle(), &self.assets, x, y);
    }

    #[inline]
    pub fn atlas_v<P: Into<Vector2>>(&mut self, pos: P) {
        let pos: Vector2 = pos.into();

        self.atlas(pos.x, pos.y);
    }
}
