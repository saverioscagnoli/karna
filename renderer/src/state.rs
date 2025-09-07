use err::RendererError;
use std::{rc::Rc, sync::Arc};
use wgpu::Backends;
use winit::window::Window;

pub struct GpuState {
    pub surface: wgpu::Surface<'static>,
    pub device: Rc<wgpu::Device>,
    pub queue: Rc<wgpu::Queue>,
    pub adapter: wgpu::Adapter,
    pub surface_format: wgpu::TextureFormat,
}

impl GpuState {
    pub async fn new(
        window: Arc<Window>,
    ) -> Result<(Self, wgpu::SurfaceConfiguration), RendererError> {
        let (width, height) = window.inner_size().into();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::default(),
                label: Some("device"),
                required_features: wgpu::Features::MULTI_DRAW_INDIRECT, // Add this feature
                ..Default::default()
            })
            .await?;

        let device = Rc::new(device);
        let queue = Rc::new(queue);

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::default(),
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Ok((
            Self {
                surface,
                device,
                queue,
                adapter,
                surface_format,
            },
            config,
        ))
    }
}
