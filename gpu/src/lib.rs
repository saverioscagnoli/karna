mod surface;

use std::sync::OnceLock;

use logging::debug;
use wgpu::InstanceDescriptor;

pub use crate::surface::WindowSurface;

static SINGLETON: OnceLock<GpuState> = OnceLock::new();

#[derive(Debug)]
pub struct GpuState {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuState {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get() -> &'static Self {
        SINGLETON.get_or_init(|| pollster::block_on(Self::new()))
    }

    /// On the web the singleton cannot be created lazily (that would require
    /// blocking), so it must have been set by awaiting [`init_async`] first.
    #[cfg(target_arch = "wasm32")]
    pub fn get() -> &'static Self {
        SINGLETON
            .get()
            .expect("GpuState::get() called before gpu::init_async() completed")
    }

    async fn new() -> Self {
        let backend_options = wgpu::BackendOptions::default();

        debug!(
            "Creating instance with backend options {:?}",
            backend_options
        );

        let instance =
            wgpu::Instance::new(InstanceDescriptor::new_without_display_handle_from_env());

        #[cfg(not(target_arch = "wasm32"))]
        debug!(
            "adapters {:?}",
            instance
                .enumerate_adapters(wgpu::Backends::all())
                .await
                .to_vec()
        );

        // Will be parsed from a configuration file maybe?
        let power_preference = wgpu::PowerPreference::HighPerformance;

        debug!(
            "Requesting adapter with power preference {:?}",
            power_preference
        );

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .expect("Failed to request adapter");

        // Will be parsed from a configuration file maybe?
        let required_limits = wgpu::Limits::default();
        let required_features = wgpu::Features::default();

        debug!("Requesting device features={:?}", required_features);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_limits,
                required_features,
                ..Default::default()
            })
            .await
            .expect("Failed to request device");

        Self {
            instance,
            adapter,
            device,
            queue,
        }
    }
}
