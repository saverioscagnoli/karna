use math::Vector4;
use utils::Handle;

use crate::Camera;
use crate::Color;
use crate::Layer;
use crate::LayerId;
use crate::Projection;
use crate::Renderer;
use crate::retained::mesh::Mesh;

pub struct SceneHandle<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> SceneHandle<'a> {
    #[doc(hidden)]
    pub fn _new(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.renderer.active_layer = LayerId(layer as usize)
    }

    pub fn clear_color(&self) -> Color {
        self.renderer.clear_color.into()
    }

    pub fn set_clear_color<C: Into<Vector4<f32>>>(&mut self, color: C) {
        self.renderer.clear_color = color.into()
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.renderer.active_layer_mut().retained.add_mesh(mesh)
    }

    pub fn mesh(&self, mesh: Handle<Mesh>) -> &Mesh {
        self.renderer.active_layer().retained.mesh(mesh)
    }

    pub fn mesh_mut(&mut self, mesh: Handle<Mesh>) -> &mut Mesh {
        self.renderer.active_layer_mut().retained.mesh_mut(mesh)
    }

    pub fn remove_mesh(&mut self, mesh: Handle<Mesh>) -> Mesh {
        self.renderer.active_layer_mut().retained.remove_mesh(mesh)
    }

    pub fn camera(&self) -> &Camera {
        &self.renderer.active_layer().camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.renderer.active_layer_mut().camera
    }

    pub fn camera_projection(&mut self) -> &Projection {
        &self.camera().projection
    }

    pub fn camera_projection_mut(&mut self) -> &mut Projection {
        &mut self.camera_mut().projection
    }

    pub fn set_camera_projection(&mut self, proj: Projection) {
        self.camera_mut().projection = proj;
    }
}
