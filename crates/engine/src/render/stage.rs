use std::rc::Rc;

use crate::Camera;
use crate::Draw;
use crate::Projection;
use crate::gpu::Gpu;
use crate::render::draw::DrawState;
use crate::render::geometry::ImmediateGeometry;
use crate::render::layer::Layer;
use crate::render::layer::LayerKind;
use crate::render::layer::LayerMap;

pub struct Stage {
    cameras: LayerMap<Camera>,
    data: LayerMap<LayerKind>,
}

impl Stage {
    pub fn new(gpu: Rc<Gpu>, viewport: math::Size<u32>) -> Self {
        let default_camera = Camera::new(Projection::topleft_2d(viewport));
        let cameras = LayerMap::new(
            default_camera.clone(),
            default_camera.clone(),
            default_camera,
        );

        let data = LayerMap::new(
            LayerKind::Mesh {
                immediate: ImmediateGeometry::new(
                    gpu.clone(),
                    "world immediate vertex buffer",
                    Vec::new(),
                    Vec::new(),
                ),
            },
            LayerKind::Content {
                immediate: ImmediateGeometry::new(
                    gpu.clone(),
                    "ui immediate vertex buffer",
                    Vec::new(),
                    Vec::new(),
                ),
            },
            LayerKind::Content {
                immediate: ImmediateGeometry::new(
                    gpu.clone(),
                    "debug immediate vertex buffer",
                    Vec::new(),
                    Vec::new(),
                ),
            },
        );

        Self { cameras, data }
    }

    pub fn scene_view(&mut self) -> SceneView<'_> {
        SceneView {
            cameras: &mut self.cameras,
            data: &mut self.data,
        }
    }

    pub fn draw<'a>(&'a mut self, state: &'a mut DrawState, viewport: math::Size<u32>) -> Draw<'a> {
        Draw {
            state,
            cameras: &self.cameras,
            data: &mut self.data,
            viewport,
        }
    }
}

pub struct SceneView<'a> {
    cameras: &'a mut LayerMap<Camera>,
    data: &'a mut LayerMap<LayerKind>,
}

impl<'a> SceneView<'a> {
    pub fn camera(&self, layer: Layer) -> &Camera {
        &self.cameras[layer]
    }

    pub fn camera_mut(&mut self, layer: Layer) -> &mut Camera {
        &mut self.cameras[layer]
    }
}
