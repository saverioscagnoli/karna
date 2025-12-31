pub mod core;
mod texture;

use macros::Get;
use std::sync::OnceLock;

// Re-exports
pub use texture::Texture;

static STATE: OnceLock<GpuState> = OnceLock::new();

pub fn init() {
    STATE
        .set(pollster::block_on(GpuState::new()))
        .expect("Failed to initialize gpu");
}

#[inline]
pub fn get() -> &'static GpuState {
    STATE.get().expect("Failed to get gpu")
}

#[inline]
pub fn adapter() -> &'static wgpu::Adapter {
    &get().adapter
}

#[inline]
pub fn device() -> &'static wgpu::Device {
    &get().device
}

#[inline]
pub fn queue() -> &'static wgpu::Queue {
    &get().queue
}

#[derive(Debug)]
#[derive(Get)]
pub struct GpuState {
    #[get]
    instance: wgpu::Instance,

    #[get]
    adapter: wgpu::Adapter,

    #[get]
    device: wgpu::Device,

    #[get]
    queue: wgpu::Queue,
}

impl GpuState {
    async fn new() -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
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
                required_features: wgpu::Features::default()
                    .union(wgpu::Features::POLYGON_MODE_LINE),
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
