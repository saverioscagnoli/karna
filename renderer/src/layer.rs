use assets::AssetServerView;
use assets::ReadOnly;
use gpu::PipelineCache;

use crate::ImmediateRenderer;
use crate::camera::Camera;
use crate::retained::RetainedRenderer;

#[repr(usize)]
pub enum Layer {
    World = 0,
    Ui = 1,
    Debug = 2,
}

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

    pub fn present<'rp, 'assets>(
        &mut self,
        view: math::Size<u32>,
        rp: &mut wgpu::RenderPass<'rp>,
        pipelines: &PipelineCache,
        assets: &AssetServerView<'assets, ReadOnly>,
    ) {
        self.camera.update(view);
        self.immediate.present(rp, pipelines);
        self.retained.present(rp, pipelines, assets);
    }
}
