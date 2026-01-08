use crate::{
    camera::Camera,
    immediate::ImmediateRenderer,
    retained::{RetainedRenderer, TextRenderer},
};
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
    pub(crate) text: TextRenderer,
}

impl RenderLayer {
    pub(crate) fn new(
        config: &wgpu::SurfaceConfiguration,
        assets: &AssetServer,
        camera: Camera,
    ) -> Self {
        let immediate = ImmediateRenderer::new(config.format, &camera, &assets);
        let retained = RetainedRenderer::new(config.format, &camera, &assets);
        let text = TextRenderer::new(config.format, &camera, &assets);

        Self {
            camera,
            retained,
            immediate,
            text,
        }
    }

    #[inline]
    pub fn queue_resize(&mut self) {
        self.camera.queue_resize();
    }

    #[inline]
    pub fn present<'a>(
        &'a mut self,
        view: Size<u32>,
        render_pass: &mut wgpu::RenderPass<'a>,
        assets: &AssetServer,
    ) {
        self.camera.update(view);

        self.immediate.present(render_pass);
        self.retained.present(render_pass, assets);
        self.text.present(render_pass, assets);
    }
}
