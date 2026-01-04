use crate::{Layer, Renderer, color::Color, retained::SceneView};
use assets::{AssetServer, Font, Image};
use macros::{Get, Set};
use utils::Handle;

#[derive(Get, Set)]
pub struct Draw<'a> {
    #[get(prop = "clear_color", ty = &Color, name = "clear_color")]
    #[get(mut, prop = "clear_color", ty = &Color, name = "clear_color_mut")]
    #[set(into, prop = "clear_color", ty = Color, name = "set_clear_color")]
    #[get(prop = "active_layer", ty = &Layer, name = "layer")]
    #[set(prop = "active_layer", ty = Layer, name = "set_layer")]
    renderer: &'a mut Renderer,
    assets: &'a AssetServer,
}

impl<'a> Draw<'a> {
    #[doc(hidden)]
    pub fn new(renderer: &'a mut Renderer, assets: &'a AssetServer) -> Self {
        Self { renderer, assets }
    }

    #[inline]
    pub fn scene(&self) -> SceneView<'_> {
        SceneView::new(self.renderer)
    }

    #[inline]
    pub fn color(&self) -> &Color {
        &self
            .renderer
            .layer(self.renderer.active_layer)
            .immediate
            .draw_color
    }

    #[inline]
    pub fn color_mut(&mut self) -> &mut Color {
        &mut self
            .renderer
            .layer_mut(self.renderer.active_layer)
            .immediate
            .draw_color
    }

    #[inline]
    pub fn set_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.renderer
            .layer_mut(self.renderer.active_layer)
            .immediate
            .draw_color = color.into()
    }

    #[inline]
    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        layer.immediate.fill_rect([x, y].into(), w, h, &self.assets);
    }

    #[inline]
    pub fn image(&mut self, image: Handle<Image>, x: f32, y: f32) {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        layer
            .immediate
            .draw_image(image, [x, y].into(), &self.assets);
    }

    #[inline]
    pub fn text<T: AsRef<str>>(&mut self, font: Handle<Font>, text: T, x: f32, y: f32) {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        layer
            .immediate
            .draw_text(font, text.as_ref(), x, y, &self.assets);
    }

    #[inline]
    pub fn debug_text<T: AsRef<str>>(&mut self, text: T, x: f32, y: f32) {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        layer
            .immediate
            .draw_text(self.assets.debug_font(), text.as_ref(), x, y, &self.assets);
    }
}
