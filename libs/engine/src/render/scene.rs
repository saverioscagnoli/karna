use core::ops::Deref;

use crate::event::AppEvent;
use crate::event::Outbox;
use crate::render::Camera;
use crate::render::Layer;
use crate::render::LayerMap;
use crate::render::Projection;

pub struct SceneData {
    cameras: LayerMap<Camera>,
}

impl SceneData {
    pub(crate) fn new(viewport: math::Size<u32>) -> Self {
        let default_camera = Camera::new(Projection::topleft_ortho(viewport));
        let cameras = LayerMap::new(default_camera, default_camera, default_camera);

        Self { cameras }
    }

    pub(crate) fn sync(&mut self, viewport: math::Size<u32>) {
        for camera in self.cameras.values_mut() {
            camera.update(viewport);
        }
    }

    pub fn camera(&self, layer: Layer) -> &Camera {
        if self.cameras.contains(layer) {
            &self.cameras[layer]
        } else {
            &self.cameras[Layer::WORLD]
        }
    }
}

pub struct SceneHandle<'a> {
    pub(crate) data: &'a mut SceneData,
    pub(crate) outbox: &'a mut Outbox<AppEvent>,
}

impl Deref for SceneHandle<'_> {
    type Target = SceneData;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl SceneHandle<'_> {
    pub fn camera_mut(&mut self, layer: Layer) -> &mut Camera {
        let cameras = &mut self.data.cameras;

        if !cameras.contains(layer) {
            let camera = cameras[Layer::WORLD];
            cameras.insert(layer, camera);
        }

        &mut cameras[layer]
    }

    pub fn set_camera(&mut self, layer: Layer, camera: Camera) {
        self.data.cameras.insert(layer, camera);
    }
}
