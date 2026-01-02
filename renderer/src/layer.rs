use crate::{camera::Camera, immediate::ImmediateRenderer, retained::RetainedRenderer};
use assets::AssetManager;
use std::sync::Arc;

#[derive(Debug, Clone, Copy)]
pub enum Layer {
    World,
    Ui,
    N(usize),
}

pub struct RenderLayer {
    pub(crate) camera: Camera,

    assets: Arc<AssetManager>,

    pub(crate) immediate: ImmediateRenderer,
    pub(crate) retained: RetainedRenderer,
}

impl RenderLayer {
    pub(crate) fn new(camera: Camera, assets: Arc<AssetManager>) -> Self {
        Self {
            camera,
            assets: assets.clone(),
            immediate: ImmediateRenderer::new(assets.clone()),
            retained: RetainedRenderer::new(assets),
        }
    }

    #[inline]
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width, height);
    }

    #[inline]
    pub(crate) fn update(&mut self, width: u32, height: u32, dt: f32) {
        if self.camera.dirty() {
            self.camera.resize(width, height);
        }

        self.camera.update_shake(dt);
    }

    #[inline]
    pub(crate) fn present<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        retained_pipeline: &'a wgpu::RenderPipeline,
        immediate_pipeline: &'a wgpu::RenderPipeline,
        immediate_line_pipeline: &'a wgpu::RenderPipeline,
        text_pipeline: &'a wgpu::RenderPipeline,
    ) {
        self.retained
            .present(render_pass, retained_pipeline, text_pipeline);

        self.immediate
            .present(render_pass, immediate_pipeline, immediate_line_pipeline);
    }
}
