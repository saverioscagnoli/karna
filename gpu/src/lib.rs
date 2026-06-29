mod buffer;
mod pipeline;
mod shaders;

use std::sync::OnceLock;

pub use crate::buffer::Buffer;
pub use crate::pipeline::PipelineCache;
pub use crate::pipeline::PipelineDesc;
use crate::shaders::ShaderStore;

static SINGLETON: OnceLock<GpuState> = OnceLock::new();

pub fn init(f: impl FnOnce(&mut ShaderStore, &wgpu::Device)) {
    SINGLETON.get_or_init(|| {
        let mut state = pollster::block_on(GpuState::new());
        f(&mut state.shaders, &state.device);
        state
    });
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
    pub fn get() -> &'static Self {
        SINGLETON.get_or_init(|| pollster::block_on(Self::new()))
    }

    async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            backend_options: wgpu::BackendOptions::default(),
            display: None,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to request adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::defaults(),
                label: Some("device"),
                required_features: wgpu::Features::default(),
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
