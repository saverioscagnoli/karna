use gpu::GpuState;
use logging::info;
use utils::ByteSize;
use utils::Handle;
use utils::SlotMap;

#[derive(Debug, Clone)]
pub struct Image {
    pub data: Vec<u8>,
    pub size: math::Size<u32>,
    pub uv: math::Vector4<f32>,
}

pub struct TextureAtlas {
    pub(super) size: math::Size<u32>,
    pub(super) texture: gpu::Texture,
    packer: rect_packer::DensePacker,
    pub(super) images: SlotMap<Image>,

    pub(super) white: Handle<Image>,
    pub(super) handle: Handle<Image>,
}

impl TextureAtlas {
    pub fn new<S: Into<math::Size<u32>>>(size: S, atlas_layout: &wgpu::BindGroupLayout) -> Self {
        let size: math::Size<u32> = size.into();
        let gpu = GpuState::get();

        let mut images = SlotMap::new();
        let mut packer = rect_packer::DensePacker::new(size.width as i32, size.height as i32);
        let texture = gpu::Texture::new_empty("texture atlas", size, atlas_layout, &gpu.device);

        let white_region = packer.pack(1, 1, false).expect("Failed to pack");
        let white_handle = images.insert(Image {
            data: vec![255, 255, 255, 255],
            size: math::Size::new(1, 1),
            uv: image_uv(&size, white_region),
        });

        let atlas_region = rect_packer::Rect::new(0, 0, size.width as i32, size.height as i32);
        let atlas_handle = images.insert(Image {
            data: vec![255, 255, 255, 255],
            size,
            uv: image_uv(&size, atlas_region),
        });

        // Write it directly, otherwise it will misalign the white pixel coordinates
        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: texture.wgpu(),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: white_region.x as u32,
                    y: white_region.y as u32,
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

        Self {
            size,
            texture,
            packer,
            images,

            white: white_handle,
            handle: atlas_handle,
        }
    }

    fn write_region_gpu(&mut self, data: &[u8], size: math::Size<u32>) -> rect_packer::Rect {
        let region = self
            .packer
            .pack(size.width as i32, size.height as i32, false)
            .expect("Failed to pack");

        let gpu = GpuState::get();

        gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: self.texture.wgpu(),
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

    pub fn load_raw(
        &mut self,
        data: Vec<u8>,
        size: math::Size<u32>,
        is_glyph: bool,
    ) -> Handle<Image> {
        let region = self.write_region_gpu(&data, size);
        let size_bytes = data.len() as u64;
        let handle = self.images.insert(Image {
            data,
            size,
            uv: image_uv(&self.size, region),
        });

        if !is_glyph {
            info!(
                "Loaded image with size={}",
                ByteSize::from_bytes(size_bytes)
            );
        }

        handle
    }

    // Existing: for real PNG file bytes
    pub fn load_image(&mut self, bytes: &[u8]) -> Handle<Image> {
        let image = image::decode_png(bytes).expect("Failed to decode png");
        self.load_raw(
            image.pixels,
            math::Size::new(image.width, image.height),
            false,
        )
    }
}

pub fn image_uv(atlas_size: &math::Size<u32>, region: rect_packer::Rect) -> math::Vector4<f32> {
    math::Vector4::new(
        region.x as f32 / atlas_size.width as f32,
        region.y as f32 / atlas_size.height as f32,
        region.width as f32 / atlas_size.width as f32,
        region.height as f32 / atlas_size.height as f32,
    )
}
