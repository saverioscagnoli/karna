use arc_swap::ArcSwap;
use common::utils::Label;
use math::Size;
use std::sync::{Arc, OnceLock};
use wgpu::{Backends, naga::FastHashMap};

static GPU: OnceLock<GPU> = OnceLock::new();

pub fn gpu() -> &'static GPU {
    GPU.get()
        .expect("Trying to get gpu while not being initialized")
}

#[derive(Debug)]
pub struct GPU {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl GPU {
    #[doc(hidden)]
    pub async fn init() {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::PRIMARY,
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
                required_features: wgpu::Features::default(),
                ..Default::default()
            })
            .await
            .expect("Failed to request device");

        GPU.set(Self {
            instance,
            adapter,
            device,
            queue,
        })
        .expect("Failed to set gpu");
    }

    #[inline]
    /// Returns information about the GPU adapter.
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }
}
