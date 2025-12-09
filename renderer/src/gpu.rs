use crate::{TextureAtlas, text::Font};
use common::utils::Label;
use math::Size;
use std::sync::{Arc, RwLock};
use wgpu::{Backends, naga::FastHashMap};

#[derive(Debug)]
pub struct GPU {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub texture_atlas: RwLock<TextureAtlas>,
    pub fonts: RwLock<FastHashMap<Label, Arc<Font>>>,
}

impl GPU {
    #[doc(hidden)]
    pub async fn init() -> Self {
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

        let texture_atlas = RwLock::new(TextureAtlas::new(&device, &queue, Size::new(1024, 1024)));
        let fonts = RwLock::new(FastHashMap::default());

        Self {
            instance,
            adapter,
            device,
            queue,
            texture_atlas,
            fonts,
        }
    }

    #[inline]
    /// Returns information about the GPU adapter.
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }
}
