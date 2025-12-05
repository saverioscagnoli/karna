use crate::{AtlasRegion, TextureAtlas, material::Texture};
use common::utils::Label;
use math::Size;
use std::sync::{Arc, RwLock};
use wgpu::Backends;

/// Global GPU instance represented for all windows.
/// It is imperative that the struct will not take &mut self, or will be mutated
/// in any way that could cause data races.
/// That's why atlas is wrapped in RwLock
pub struct GPU {
    pub(crate) instance: wgpu::Instance,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) default_white_texture: Arc<Texture>,
    pub atlas: RwLock<TextureAtlas>,
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(
            1,
            1,
            image::Rgba([255, 255, 255, 255]),
        ));

        let default_white_texture = Arc::new(
            Texture::from_image(
                &device,
                &queue,
                &img,
                Some("default white"),
                &texture_bind_group_layout,
            )
            .expect("Failed to create default white texture"),
        );

        let atlas = RwLock::new(TextureAtlas::new(&device, Size::new(1000, 1000)));

        Self {
            instance,
            adapter,
            device,
            queue,
            texture_bind_group_layout,
            default_white_texture,
            atlas,
        }
    }

    #[inline]
    pub fn info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    #[inline]
    pub fn load_texture(&self, label: Label, bytes: &[u8]) -> Result<AtlasRegion, String> {
        self.atlas
            .write()
            .unwrap()
            .load_image(&self.queue, label, bytes)
    }
}
