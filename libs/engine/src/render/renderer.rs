use sdl3::gpu::Device;

use crate::render::Camera;
use crate::render::LayerData;
use crate::render::LayerMap;
use crate::render::Projection;

pub struct Renderer {
    device: Device,
    cameras: LayerMap<Camera>,
    data: LayerMap<LayerData>,
}

impl Renderer {
    pub fn new(device: Device, viewport: math::Size<u32>) -> Self {
        let default_camera = Camera::new(Projection::topleft_ortho(viewport));
        let cameras = LayerMap::new(default_camera, default_camera, default_camera);
        let data = LayerMap::new(LayerData {}, LayerData {}, LayerData {});

        Self {
            device,
            cameras,
            data,
        }
    }
}
