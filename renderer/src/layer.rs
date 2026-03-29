use macros::Get;
use math::Size;

use crate::Camera;
use crate::ImmediateRenderer;
use crate::OrthographicProjection;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ActiveLayer {
    #[default]
    World,
    Ui,
    Custom(usize),
}

#[derive(Get)]
pub struct RenderLayer {
    camera: Camera,
    #[get]
    #[get(mut)]
    immediate: ImmediateRenderer,
}

impl RenderLayer {
    pub fn new(view: Size<u32>, surface_format: wgpu::TextureFormat) -> Self {
        let camera = Camera::new(OrthographicProjection::new(
            0.0,
            view.width as f32,
            view.height as f32,
            0.0,
            -1.0,
            1.0,
        ));

        // Ensure the view-projection uniform buffer contains a valid matrix before the first draw.
        // If it's left uninitialized (or all zeros), every vertex will be transformed to clip-space
        // incorrectly and get clipped, so you'll only see the clear color.
        camera.update(view);

        Self {
            immediate: ImmediateRenderer::new(surface_format, &camera.vp_bgl()),
            camera,
        }
    }

    #[inline]
    pub fn update(&mut self, view: Size<u32>) {
        self.camera.set_projection(OrthographicProjection::new(
            0.0,
            view.width as f32,
            view.height as f32,
            0.0,
            -1.0,
            1.0,
        ));
        self.camera.update(view);
    }

    #[inline]
    pub fn flush<'pass>(&'pass mut self, render_pass: &mut wgpu::RenderPass<'pass>) {
        // The immediate pipeline expects the camera bind group at index 0:
        //   @group(0) @binding(0) var<uniform> view_projection: mat4x4<f32>;
        render_pass.set_bind_group(0, self.camera.vp_bg(), &[]);

        self.immediate.present(render_pass);
    }
}
