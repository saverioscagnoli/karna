use std::rc::Rc;

pub struct RendererConfig {
    surface: Rc<wgpu::Surface<'static>>,
    device: Rc<wgpu::Device>,
    wgpu_config: wgpu::SurfaceConfiguration,
    vsync: bool,
}

impl RendererConfig {
    pub fn new(
        surface: Rc<wgpu::Surface<'static>>,
        device: Rc<wgpu::Device>,
        wgpu_config: wgpu::SurfaceConfiguration,
    ) -> Self {
        Self {
            surface,
            device,
            wgpu_config,
            vsync: true,
        }
    }

    pub(crate) fn wgpu(&self) -> &wgpu::SurfaceConfiguration {
        &self.wgpu_config
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.wgpu_config.width = width;
        self.wgpu_config.height = height;

        self.surface.configure(&self.device, &self.wgpu_config);
    }

    pub fn vsync(&self) -> bool {
        self.vsync
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        self.vsync = vsync;
        self.wgpu_config.present_mode = if vsync {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::Immediate
        };

        self.surface.configure(&self.device, &self.wgpu_config);
    }
}
