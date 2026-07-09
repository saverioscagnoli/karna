use crate::GpuState;

pub struct WindowSurface {
    inner: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl WindowSurface {
    pub fn create<S, V>(gpu: &GpuState, surface: S, size: V) -> Self
    where
        S: Into<wgpu::SurfaceTarget<'static>>,
        V: Into<math::Size<u32>>,
    {
        let size: math::Size<u32> = size.into();

        let surface = gpu
            .instance
            .create_surface(surface.into())
            .expect("Failed to create surface");

        let capabilities = surface.get_capabilities(&gpu.adapter);
        let format = capabilities
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(capabilities.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            // Default to no vsync, may change later
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        Self {
            inner: surface,
            config,
        }
    }

    pub fn resize(&mut self, gpu: &GpuState, size: math::Size<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;

        self.inner.configure(&gpu.device, &self.config);
    }
}
