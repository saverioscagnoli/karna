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

        let present_mode = if capabilities
            .present_modes
            .contains(&wgpu::PresentMode::Mailbox)
        {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::AutoNoVsync
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            // A zero-sized surface is invalid to configure. The web reports
            // the canvas size asynchronously (0x0 until the ResizeObserver
            // fires) and native windows can spawn minimized; start at 1x1
            // and let the first Resized event reconfigure.
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        surface.configure(&gpu.device, &config);

        Self {
            inner: surface,
            config,
        }
    }

    pub fn resize(&mut self, gpu: &GpuState, size: math::Size<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        if self.config.width == size.width && self.config.height == size.height {
            return; // already this size — don't reconfigure
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.inner.configure(&gpu.device, &self.config);
    }

    pub fn acquire(&mut self) -> wgpu::CurrentSurfaceTexture {
        self.inner.get_current_texture()
    }
}
