use crate::{
    Camera, Color, Renderer,
    retained::{RetainedRenderer, mesh::Mesh},
};
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
    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        layer.retained.add_mesh(mesh)
    }

    #[inline]
    pub fn get_mesh(&self, handle: Handle<Mesh>) -> Option<&Mesh> {
        let layer = self.renderer.layer(self.renderer.active_layer);

        layer.retained.get_mesh(handle)
    }

    #[inline]
    pub fn get_mesh_mut(&mut self, handle: Handle<Mesh>) -> Option<&mut Mesh> {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        layer.retained.get_mesh_mut(handle)
    }

    #[inline]
    pub fn remove_mesh(&mut self, handle: Handle<Mesh>) {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        layer.retained.remove_mesh(handle)
    }

    #[inline]
    pub fn retained(&mut self) -> &mut RetainedRenderer {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        &mut layer.retained
    }

    #[inline]
    pub fn camera(&self) -> &Camera {
        let layer = self.renderer.layer(self.renderer.active_layer);

        &layer.camera
    }

    #[inline]
    pub fn camera_mut(&mut self) -> &mut Camera {
        let layer = self.renderer.layer_mut(self.renderer.active_layer);

        &mut layer.camera
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
        let layer = self.renderer.layer(self.renderer.active_layer);

        layer.retained.get_mesh(handle)
    }

    #[inline]
    pub fn get_camera(&self) -> &Camera {
        let layer = self.renderer.layer(self.renderer.active_layer);

        &layer.camera
    }
}
