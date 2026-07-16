use logging::debug;

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// Screen-sized depth attachment. Recreated whenever the surface is resized.
pub struct DepthTexture {
    #[allow(unused)]
    inner: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: math::Size<u32>,
}

impl DepthTexture {
    pub fn new(size: math::Size<u32>) -> Self {
        let gpu = crate::GpuState::get();

        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: size.width.max(1),
                height: size.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        debug!("Creating depth texture {:?}", size);

        Self {
            inner: texture,
            view,
            size,
        }
    }
}

#[allow(unused)]
pub struct Texture {
    inner: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,

    pub bind_group: wgpu::BindGroup,

    pub size: math::Size<u32>,
}

impl Texture {
    pub fn new_empty<L: AsRef<str>>(
        label: L,
        size: math::Size<u32>,
        bind_group_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
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
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label.as_ref()),
            layout: bind_group_layout,
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

    pub fn new<L: AsRef<str>>(
        label: L,
        size: math::Size<u32>,
        data: &[u8],
        bind_group_layout: &wgpu::BindGroupLayout,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label.as_ref()),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Upload the initial pixel data immediately after creation
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width), // 4 bytes/pixel for Rgba8
                rows_per_image: Some(size.height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label.as_ref()),
            layout: bind_group_layout,
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

        debug!(
            "Creating new texture with data '{}' {:?}",
            label.as_ref(),
            size
        );

        Self {
            inner: texture,
            view,
            sampler,
            bind_group,
            size,
        }
    }

    pub fn wgpu(&self) -> &wgpu::Texture {
        &self.inner
    }
}
