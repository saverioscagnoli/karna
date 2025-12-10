use crate::{MeshGeometry, TextureAtlas, text::Font};
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
    pub(crate) texture_atlas: ArcSwap<TextureAtlas>,
    pub(crate) fonts: ArcSwap<FastHashMap<Label, Arc<Font>>>,

    // Caches
    pub(crate) geometry_cache: ArcSwap<FastHashMap<u32, Arc<MeshGeometry>>>,
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

        let texture_atlas =
            ArcSwap::from_pointee(TextureAtlas::new(&device, &queue, Size::new(1024, 1024)));

        let fonts = ArcSwap::from_pointee(FastHashMap::default());
        let geometry_cache = ArcSwap::from_pointee(FastHashMap::default());

        GPU.set(Self {
            instance,
            adapter,
            device,
            queue,
            texture_atlas,
            fonts,
            geometry_cache,
        })
        .expect("Failed to set gpu");
    }

    #[inline]
    /// Returns information about the GPU adapter.
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }
}
