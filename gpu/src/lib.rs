mod buffer;
mod pipeline;
mod shaders;
mod surface;
mod texture;
mod vertex;

use std::sync::OnceLock;

use logging::debug;
pub use wgpu; // re-export so engine can write gpu::wgpu::Device WASM ONLY

pub use crate::buffer::Buffer;
pub use crate::pipeline::PipelineCache;
pub use crate::pipeline::PipelineDesc;
pub use crate::shaders::ShaderStore;
pub use crate::surface::WindowSurface;
pub use crate::texture::Texture;
pub use crate::vertex::*;

static SINGLETON: OnceLock<GpuState> = OnceLock::new();

#[cfg(not(target_arch = "wasm32"))]
pub fn init(f: impl FnOnce(&mut ShaderStore, &wgpu::Device)) {
    SINGLETON.get_or_init(|| {
        let mut state = pollster::block_on(GpuState::new());
        f(&mut state.shaders, &state.device);
        state
    });
}

/// Async variant of [`init`] — the only option on the web, where the
/// adapter/device requests must actually be awaited instead of blocked on.
pub async fn init_async(f: impl FnOnce(&mut ShaderStore, &wgpu::Device)) {
    if SINGLETON.get().is_some() {
        return;
    }

    let mut state = GpuState::new().await;
    f(&mut state.shaders, &state.device);

    _ = SINGLETON.set(state);
}

#[derive(Debug)]
pub struct GpuState {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub shaders: ShaderStore,
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

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            backend_options,
            display: None,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });
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
            shaders: ShaderStore::new(),
        }
    }
}
