use crate::{Color, Renderer, retained::mesh::Mesh};
use macros::{Get, Set};
use utils::Handle;

#[derive(Get, Set)]
pub struct Scene<'a> {
    #[get(prop = "clear_color", ty = &Color, name = "clear_color")]
    #[get(mut, prop = "clear_color", ty = &Color, name = "clear_color_mut")]
    #[set(into, prop = "clear_color", ty = Color, name = "set_clear_color")]
    renderer: &'a mut Renderer,
}

impl<'a> Scene<'a> {
    #[doc(hidden)]
    pub fn new(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    #[inline]
    pub fn get_mesh(&self, handle: Handle<Mesh>) -> Option<&Mesh> {
        self.renderer
            .layer(self.renderer.active_layer)
            .retained
            .get_mesh(handle)
    }

    #[inline]
    pub fn get_mesh_mut(&mut self, handle: Handle<Mesh>) -> Option<&mut Mesh> {
        self.renderer
            .layer_mut(self.renderer.active_layer)
            .retained
            .get_mesh_mut(handle)
    }
}

pub struct SceneView<'a> {
    renderer: &'a Renderer,
}

impl<'a> SceneView<'a> {
    #[doc(hidden)]
    pub fn new(renderer: &'a Renderer) -> Self {
        Self { renderer }
    }

    #[inline]
    pub fn get_mesh(&self, handle: Handle<Mesh>) -> Option<&Mesh> {
        self.renderer
            .layer(self.renderer.active_layer)
            .retained
            .get_mesh(handle)
    }
}
