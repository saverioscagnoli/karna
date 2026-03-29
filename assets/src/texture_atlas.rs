use math::Size;
use utils::FastHashMap;

pub struct TextureAtlas {
    size: Size<u32>,
    texture: gpu::Texture,
    bgl: wgpu::BindGroupLayout,
    regions: FastHashMap<String, rect_packer::Rect>,
    packer: rect_packer::DensePacker,
}

impl TextureAtlas {
    pub fn new<S: Into<Size<u32>>>(size: S) -> Self {
        let size: Size<u32> = size.into();
        let device = gpu::device();
        let queue = gpu::queue();

        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Texture atlas Bind Group Layout"),
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

        let texture = gpu::Texture::new_empty("Texture Atlas", size, &bgl, device);
        let mut packer = rect_packer::DensePacker::new(size.width as i32, size.height as i32);

        let white_pixel = packer
            .pack(1, 1, false)
            .expect("Failed to pack white pixel");

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: texture.inner(),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: white_pixel.x as u32,
                    y: white_pixel.y as u32,
                    z: 0,
                },
            },
            &[255, 255, 255, 255],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let mut regions = FastHashMap::default();

        regions.insert(String::from("_white"), white_pixel);

        regions.insert(
            String::from("_atlas"),
            rect_packer::Rect {
                x: 0,
                y: 0,
                width: size.width as i32,
                height: size.height as i32,
            },
        );

        Self {
            texture,
            bgl,
            size,
            packer,
            regions,
        }
    }
}
