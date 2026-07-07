use utils::Handle;

use crate::Renderer;
use crate::retained::mesh::Mesh;

pub struct SceneHandle<'a> {
    renderer: &'a mut Renderer,
}

impl<'a> SceneHandle<'a> {
    pub fn _new(renderer: &'a mut Renderer) -> Self {
        Self { renderer }
    }

    pub fn add(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.renderer.active_layer_mut().retained.add(mesh)
    }

    pub fn get(&self, mesh: Handle<Mesh>) -> &Mesh {
        self.renderer.active_layer().retained.get(mesh)
    }

    pub fn get_mut(&mut self, mesh: Handle<Mesh>) -> &mut Mesh {
        self.renderer.active_layer_mut().retained.get_mut(mesh)
    }
}
