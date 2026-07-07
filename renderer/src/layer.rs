use assets::AssetServerGuard;
use gpu::PipelineCache;
use math::Size;

use crate::ImmediateRenderer;
use crate::camera::Camera;
use crate::retained::RetainedRenderer;

#[derive(Debug, Clone, Copy)]
pub struct LayerId(pub usize);

pub struct RenderLayer {
    pub(crate) camera: Camera,
    pub(crate) immediate: ImmediateRenderer,
    pub(crate) retained: RetainedRenderer,
}

impl RenderLayer {
    pub(crate) fn new(camera: Camera, transform_bgl: &wgpu::BindGroupLayout) -> Self {
        let immediate = ImmediateRenderer::new();
        let retained = RetainedRenderer::new(transform_bgl);

        Self {
            camera,
            immediate,
            retained,
        }
    }

    pub fn present<'a>(
        &'a mut self,
        view: Size<u32>,
        rp: &mut wgpu::RenderPass<'a>,
        pipelines: &PipelineCache,
        assets: &AssetServerGuard<'a>,
    ) {
        self.camera.update(view);
        self.immediate.present(rp, pipelines);
        self.retained.present(rp, pipelines, assets);
    }
}
