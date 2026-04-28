use logging::info;
use math::Size;
use math::Vector4;
use utils::ByteSize;
use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

use crate::decoding::decode_png;

#[derive(Debug, Clone)]
pub struct Image {
    label: String,
    size: Size<u32>,
}

pub struct TextureAtlas {
    size: Size<u32>,
    texture: gpu::Texture,
    bgl: wgpu::BindGroupLayout,
    packer: rect_packer::DensePacker,
    regions: FastHashMap<String, rect_packer::Rect>,
    images: SlotMap<Image>,
}

impl TextureAtlas {
    /// Bind group layout for the texture atlas texture+sampler.
    ///
    /// Expected bindings:
    /// - binding(0): `texture_2d<f32>`
    /// - binding(1): sampler
    #[inline]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bgl
    }

    /// Bind group containing the atlas texture view + sampler.
    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.texture.bind_group
    }
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
            images: SlotMap::new(),
        }
    }

    fn insert_region(&mut self, data: &[u8], size: Size<u32>) -> rect_packer::Rect {
        let region = self
            .packer
            .pack(size.width as i32, size.height as i32, false)
            .expect("Failed to pack");

        let queue = gpu::queue();

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: self.texture.inner(),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x as u32,
                    y: region.y as u32,
                    z: 0,
                },
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

        region
    }

    pub fn load_image(&mut self, bytes: &[u8]) -> Handle<Image> {
        let (data, size) = decode_png(bytes);
        let region = self.insert_region(&data, size);

        self.images.insert_with_key(|key| {
            let label = format!("img_{}", key.index());
            self.regions.insert(label.clone(), region);

            info!(
                "Loaded image with label={}, size={}",
                label,
                ByteSize::from_bytes(bytes.len() as u64)
            );

            Image { label, size }
        })
    }

    #[inline]
    pub fn get_uv_coordinates(&self, image: Handle<Image>) -> Vector4 {
        let image = self.images.get(image).expect("Failed to get image");
        let rect = self.regions.get(&image.label).expect("Failed to get rect");

        let atlas_w = self.size.width as f32;
        let atlas_h = self.size.height as f32;

        Vector4::new(
            rect.x as f32 / atlas_w,
            rect.y as f32 / atlas_h,
            rect.width as f32 / atlas_w,
            rect.height as f32 / atlas_h,
        )
    }

    #[inline]
    pub fn get_image_dimensions(&self, image: Handle<Image>) -> Size<u32> {
        let image = self.images.get(image).expect("Failed to get image");

        image.size
    }

    #[inline]
    pub fn get_white_uv_coordinates(&self) -> Vector4 {
        let rect = self.regions.get("_white").unwrap();

        let atlas_w = self.size.width as f32;
        let atlas_h = self.size.height as f32;

        Vector4::new(
            rect.x as f32 / atlas_w,
            rect.y as f32 / atlas_h,
            rect.width as f32 / atlas_w,
            rect.height as f32 / atlas_h,
        )
    }
}
