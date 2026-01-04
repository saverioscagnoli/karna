use std::sync::Arc;

use assets::AssetManager;
use macros::{Get, Set};

use crate::{Renderer, color::Color, retained::SceneView};

#[derive(Get, Set)]
pub struct Draw<'a> {
    #[get(prop = "clear_color", ty = &Color, name = "clear_color")]
    #[get(mut, prop = "clear_color", ty = &Color, name = "clear_color_mut")]
    #[set(into, prop = "clear_color", ty = Color, name = "set_clear_color")]
    renderer: &'a mut Renderer,
    assets: &'a AssetManager,
}

impl<'a> Draw<'a> {
    #[doc(hidden)]
    pub fn new(renderer: &'a mut Renderer, assets: &'a AssetManager) -> Self {
        Self { renderer, assets }
    }

    #[inline]
    pub fn scene(&self) -> SceneView<'_> {
        SceneView::new(self.renderer)
    }

    #[inline]
    pub fn draw_color(&self) -> &Color {
        &self
            .renderer
            .layer(self.renderer.active_layer)
            .immediate
            .draw_color
    }

    #[inline]
    pub fn draw_color_mut(&mut self) -> &mut Color {
        &mut self
            .renderer
            .layer_mut(self.renderer.active_layer)
            .immediate
            .draw_color
    }

    #[inline]
    pub fn set_draw_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.renderer
            .layer_mut(self.renderer.active_layer)
            .immediate
            .draw_color = color.into()
    }

    #[inline]
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        layer.immediate.fill_rect([x, y].into(), w, h, &self.assets);
    }
}
