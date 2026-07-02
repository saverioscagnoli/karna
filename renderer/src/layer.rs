use math::Size;

use crate::camera::Camera;
use crate::immediate::ImmediateRenderer;
use crate::pipeline::PipelineCache;

#[derive(Debug, Clone, Copy)]
pub struct LayerId(pub usize);

pub struct RenderLayer {
    pub(crate) camera: Camera,
    pub(crate) immediate: ImmediateRenderer,
}

impl RenderLayer {
    pub(crate) fn new(camera: Camera) -> Self {
        let immediate = ImmediateRenderer::new();

        Self { camera, immediate }
    }

    pub(crate) fn present(&mut self, view: Size<u32>, pipelines: &PipelineCache) {
        self.camera.update(view);
        let vp = self.camera.vs_params();
        self.immediate.present(pipelines, &vp);
    }
}
