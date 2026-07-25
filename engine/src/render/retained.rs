use crate::render::layer::LayerCameras;

pub struct SceneRef<'ctx> {
    pub(crate) cameras: &'ctx mut LayerCameras,
}
