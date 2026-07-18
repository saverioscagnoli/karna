use sdl3::video::Window;
use wgpu::rwh::DisplayHandle;
use wgpu::rwh::HandleError;
use wgpu::rwh::HasDisplayHandle;
use wgpu::rwh::HasWindowHandle;
use wgpu::rwh::RawDisplayHandle;
use wgpu::rwh::RawWindowHandle;
use wgpu::rwh::WindowHandle;

use crate::GpuState;

struct RawHandles {
    display: RawDisplayHandle,
    window: RawWindowHandle,
}

unsafe impl Send for RawHandles {}
unsafe impl Sync for RawHandles {}

impl HasDisplayHandle for RawHandles {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(unsafe { DisplayHandle::borrow_raw(self.display) })
    }
}

impl HasWindowHandle for RawHandles {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        Ok(unsafe { WindowHandle::borrow_raw(self.window) })
    }
}

pub struct WindowSurface {
    inner: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
}

impl WindowSurface {
    pub fn create(gpu: &GpuState, window: &Window) -> Self {
        let size: math::Size<u32> = window.size().into();

        let handles = RawHandles {
            display: window.display_handle().unwrap().as_raw(),
            window: window.window_handle().unwrap().as_raw(),
        };

        let surface = gpu
            .instance
            .create_surface(handles)
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
            present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: Vec::new(),
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
        if size.width == 0
            || size.height == 0
            || self.config.width == size.width && self.config.height == size.height
        {
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.inner.configure(&gpu.device, &self.config);
    }

    pub fn acquire(&mut self) -> wgpu::CurrentSurfaceTexture {
        self.inner.get_current_texture()
    }
}
