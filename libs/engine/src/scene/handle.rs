use core::ops::Deref;

use sdl3::window::WindowId;

use crate::commands::Outbox;
use crate::commands::SceneCommand;
use crate::render::Camera;
use crate::render::Layer;
use crate::render::LayerMap;
use crate::render::Projection;
use crate::scene::SceneId;

pub struct SceneData {
    id: SceneId,
    cameras: LayerMap<Camera>,
}

impl SceneData {
    pub(crate) fn new(id: SceneId, viewport: math::Size<u32>) -> Self {
        let default_camera = Camera::new(Projection::topleft_ortho(viewport));
        let cameras = LayerMap::new(default_camera, default_camera, default_camera);

        Self { id, cameras }
    }

    #[inline]
    pub fn id(&self) -> SceneId {
        self.id
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
    pub(crate) window_id: WindowId,
    pub(crate) outbox: &'a mut Outbox<(WindowId, SceneCommand)>,
}

impl Deref for SceneHandle<'_> {
    type Target = SceneData;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl SceneHandle<'_> {
    fn push(&mut self, command: SceneCommand) {
        self.outbox.push((self.window_id, command));
    }

    pub fn activate(&mut self, scene: SceneId) {
        self.push(SceneCommand::Activate(scene));
    }

    pub fn deactivate(&mut self, scene: SceneId) {
        self.push(SceneCommand::Deactivate(scene));
    }

    pub fn change(&mut self, to: SceneId) {
        self.push(SceneCommand::Deactivate(self.data.id));
        self.push(SceneCommand::Activate(to));
    }

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
