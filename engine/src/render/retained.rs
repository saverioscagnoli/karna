use crate::render::{
    camera::Camera,
    layer::{Layer, LayerCameras},
};

pub struct SceneRef<'ctx> {
    pub(crate) active_layer: Layer,
    pub(crate) cameras: &'ctx mut LayerCameras,
}

impl<'ctx> SceneRef<'ctx> {
    pub fn camera(&self) -> &Camera {
        &self.cameras[self.active_layer]
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.cameras[self.active_layer]
    }
}
