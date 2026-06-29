use gpu::PipelineCache;
use math::Size;

use crate::ImmediateRenderer;
use crate::camera::Camera;

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

    pub fn present<'a>(
        &'a mut self,
        view: Size<u32>,
        rp: &mut wgpu::RenderPass<'a>,
        pipelines: &PipelineCache,
    ) {
        self.camera.update(view);
        self.immediate.present(rp, pipelines);
    }
}
