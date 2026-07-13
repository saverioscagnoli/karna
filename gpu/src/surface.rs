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
        // The engine renders in linear space to an sRGB view on every
        // platform: user-facing colors are sRGB and get linearized at
        // their entry points (shaders / clear color), then the hardware
        // encodes linear->sRGB on write. WebGPU only exposes non-sRGB
        // surface formats, so configure with the non-sRGB base format and
        // register the sRGB variant as a view format — wgpu/WebGPU
        // explicitly allow srgb-suffix reinterpretation of the swapchain.
        let format = capabilities.formats[0].remove_srgb_suffix();
        let view_format = format.add_srgb_suffix();

        let view_formats = if view_format != format {
            vec![view_format]
        } else {
            // No sRGB variant exists (e.g. float formats) — render to the
            // base format directly; shaders still output linear, which is
            // what such formats store anyway.
            vec![]
        };

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
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats,
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

    /// The format render passes and pipelines must use: the sRGB view of
    /// the swapchain (falls back to the base format when no sRGB variant
    /// exists). Create the frame's TextureView with this format.
    pub fn view_format(&self) -> wgpu::TextureFormat {
        let srgb = self.config.format.add_srgb_suffix();
        if self.config.view_formats.contains(&srgb) {
            srgb
        } else {
            self.config.format
        }
    }
}
