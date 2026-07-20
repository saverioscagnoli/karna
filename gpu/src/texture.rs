use logging::debug;

use crate::GpuState;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Texture {
    inner: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,

    pub bind_group: wgpu::BindGroup,

    pub size: math::Size<u32>,
}

impl Texture {
    pub fn new_blank<L>(label: L, size: math::Size<u32>, layout: &wgpu::BindGroupLayout) -> Self
    where
        L: AsRef<str>,
    {
        let gpu = GpuState::get();
        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label.as_ref()),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label.as_ref()),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        debug!("Creating new texture '{}' {:?}", label.as_ref(), size);

        Self {
            inner: texture,
            view,
            sampler,
            bind_group,
            size,
        }
    }

    pub fn new<L>(
        label: L,
        size: math::Size<u32>,
        data: &[u8],
        layout: &wgpu::BindGroupLayout,
    ) -> Self
    where
        L: AsRef<str>,
    {
        let this = Self::new_blank(label, size, layout);

        this.write(data, math::Vector2::new(0, 0), size);
        this
    }

    pub fn write(&self, data: &[u8], origin: math::Vector2<u32>, size: math::Size<u32>) {
        let gpu = GpuState::get();

        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.inner,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: origin.x,
                    y: origin.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn wgpu(&self) -> &wgpu::Texture {
        &self.inner
    }
}
