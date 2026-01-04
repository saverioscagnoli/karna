use crate::{camera::Camera, immediate::ImmediateRenderer, retained::RetainedRenderer};
use assets::AssetManager;
use math::Size;
use std::sync::Arc;

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum Layer {
    #[default]
    World,
    Ui,
    Custom(usize),
}

pub struct RenderLayer {
    assets: Arc<AssetManager>,
    pub(crate) camera: Camera,

    pub(crate) retained: RetainedRenderer,
    pub(crate) immediate: ImmediateRenderer,
}

impl RenderLayer {
    pub(crate) fn new(
        config: &wgpu::SurfaceConfiguration,
        assets: Arc<AssetManager>,
        camera: Camera,
    ) -> Self {
        let immediate = ImmediateRenderer::new(config.format, &camera, &assets);

        Self {
            assets,
            camera,
            retained: RetainedRenderer::new(),
            immediate,
        }
    }

    #[inline]
    pub fn resize(&mut self, view: Size<u32>) {
        self.camera.resize(view);
    }

    #[inline]
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.immediate.present(render_pass);
    }
}
