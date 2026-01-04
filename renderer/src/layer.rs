use crate::{camera::Camera, immediate::ImmediateRenderer, retained::RetainedRenderer};
use assets::AssetServer;
use math::Size;

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum Layer {
    #[default]
    World,
    Ui,
    Custom(usize),
}

pub struct RenderLayer {
    pub(crate) camera: Camera,

    pub(crate) retained: RetainedRenderer,
    pub(crate) immediate: ImmediateRenderer,
}

impl RenderLayer {
    pub(crate) fn new(
        config: &wgpu::SurfaceConfiguration,
        assets: &AssetServer,
        camera: Camera,
    ) -> Self {
        let immediate = ImmediateRenderer::new(config.format, &camera, assets.atlas_bgl());
        let retained = RetainedRenderer::new(config.format, &camera, assets.atlas_bgl());

        Self {
            camera,
            retained,
            immediate,
        }
    }

    #[inline]
    pub fn resize(&mut self, view: Size<u32>) {
        self.camera.resize(view);
    }

    #[inline]
    pub fn present<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>, assets: &AssetServer) {
        self.camera.update();

        self.immediate.present(render_pass);
        self.retained.present(render_pass, assets);
    }
}
