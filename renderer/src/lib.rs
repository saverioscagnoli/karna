mod camera;
mod color;
pub mod immediate;

use std::sync::Arc;

pub use camera::Camera;
pub use camera::OrthographicProjection;
pub use camera::PerspectiveProjection;
pub use camera::Projection;
pub use color::Color;
pub use immediate::ImmediateRenderer;
use logging::info;
use math::Size;
use winit::window::Window;

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    view: Size<u32>,
    clear_color: Color,
    pub(crate) immediate: ImmediateRenderer,
}

impl Renderer {
    #[doc(hidden)]
    pub fn create_surface(
        window: Arc<Window>,
    ) -> (wgpu::Surface<'static>, wgpu::SurfaceConfiguration) {
        let gpu = gpu::get();
        let view: Size<u32> = window.inner_size().into();

        let surface = gpu
            .instance()
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let surface_caps = surface.get_capabilities(gpu.adapter());
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: view.width,
            height: view.height,
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(gpu.device(), &config);

        (surface, config)
    }

    /// Renderer creation cannot be combined with surface creation because
    /// the surface is only one, but the renderers are per-window, so
    /// `create_surface` must be called once in the main thread, while this one
    /// must be called at the start of each window thread.
    #[doc(hidden)]
    pub fn from_surface(
        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
    ) -> Self {
        let view = Size::new(surface_config.width, surface_config.height);
        let immediate = ImmediateRenderer::new(surface_config.format);

        Self {
            surface,
            config: surface_config,
            view,
            clear_color: Color::rgb(1.0 / 25.0, 1.0 / 25.0, 1.0 / 25.0),
            immediate,
        }
    }

    #[doc(hidden)]
    pub fn resize(&mut self, view: Size<u32>) {
        info!("Resizing window to {}x{}", view.width, view.height);

        self.config.width = view.width;
        self.config.height = view.height;
        self.view = view;
        self.surface.configure(gpu::device(), &self.config);
    }

    /// Returns the current viewport size.
    #[inline]
    pub fn view(&self) -> Size<u32> {
        self.view
    }

    /// Returns a mutable reference to the immediate renderer so callers
    /// (e.g. `Draw`) can push geometry before `present` flushes it.
    #[inline]
    pub fn immediate_mut(&mut self) -> &mut ImmediateRenderer {
        &mut self.immediate
    }

    #[doc(hidden)]
    pub fn present(&mut self) {
        let gpu = gpu::get();
        let output = self.surface.get_current_texture().expect("Bruh");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // Flush all immediate-mode geometry that was accumulated this frame.
            self.immediate.flush(self.view, &mut render_pass);
        }

        gpu.queue().submit(std::iter::once(encoder.finish()));
        output.present();
    }
}
