use imgui::{FontSource, MouseCursor};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use std::{rc::Rc, sync::Arc, time::Duration};
use traccia::warn;

pub struct ImguiState {
    device: Rc<wgpu::Device>,
    queue: Rc<wgpu::Queue>,
    window: Arc<winit::window::Window>,
    pub(crate) renderer: Renderer,

    #[doc(hidden)]
    pub(crate) context: imgui::Context,

    #[doc(hidden)]
    platform: WinitPlatform,
    last_cursor: Option<MouseCursor>,
}

impl ImguiState {
    pub fn new(
        window: Arc<winit::window::Window>,
        hidpi_factor: f32,
        device: Rc<wgpu::Device>,
        queue: Rc<wgpu::Queue>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let mut context = imgui::Context::create();
        let mut platform = WinitPlatform::new(&mut context);

        platform.attach_window(
            context.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );

        context.set_ini_filename(None);

        let font_size = 13.0 * hidpi_factor;

        context.io_mut().font_global_scale = 1.0 / hidpi_factor;
        context.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = RendererConfig {
            texture_format: format,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut context, &device, &queue, renderer_config);
        let last_cursor = None;

        Self {
            device,
            queue,
            window,
            renderer,
            context,
            platform,
            last_cursor,
        }
    }

    #[inline]
    #[doc(hidden)]
    pub fn handle_event(&mut self, event: &winit::event::Event<()>) {
        self.platform
            .handle_event(self.context.io_mut(), &self.window, event);
    }

    #[inline]
    #[doc(hidden)]
    pub fn update_dt(&mut self, dt: Duration) {
        self.context.io_mut().update_delta_time(dt);
    }

    pub fn render_frame<F>(&mut self, ui_builder: F)
    where
        F: FnOnce(&imgui::Ui),
    {
        if let Err(e) = self
            .platform
            .prepare_frame(self.context.io_mut(), &self.window)
        {
            warn!("Error during imgui frame preparation: {}", e);
        }

        let ui = self.context.new_frame();
        ui_builder(ui);

        self.platform.prepare_render(ui, &self.window)
    }
}
