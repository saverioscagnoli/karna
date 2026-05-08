use logging::info;
use macros::Get;
use math::Size;
use math::Vector4;
use utils::ByteSize;
use utils::Handle;
use utils::SlotMap;

use crate::decoding::decode_png;
use crate::font::Font;

#[derive(Debug, Clone)]
pub struct Image {
    pub size: Size<u32>,
    pub rgba: Vec<u8>,
    pub uv: Vector4,
}

#[derive(Get)]
pub struct TextureAtlas {
    #[get]
    size: Size<u32>,
    texture: gpu::Texture,
    bgl: wgpu::BindGroupLayout,
    packer: rect_packer::DensePacker,
    images: SlotMap<Image>,

    pub(super) white_pixel_handle: Handle<Image>,
    pub(super) handle: Handle<Image>,
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

        let mut images = SlotMap::new();

        let texture = gpu::Texture::new_empty("Texture Atlas", size, &bgl, device);
        let mut packer = rect_packer::DensePacker::new(size.width as i32, size.height as i32);

        let pixel_region = packer
            .pack(1, 1, false)
            .expect("Failed to pack white pixel");

        let pixel_handle = images.insert(Image {
            size: Size::new(1, 1),
            rgba: vec![255, 255, 255, 255],
            uv: Self::uv(&size.to_f32(), pixel_region),
        });

        let atlas_region = rect_packer::Rect::new(0, 0, size.width as i32, size.height as i32);
        let atlas_handle = images.insert(Image {
            size,
            rgba: vec![255, 255, 255, 255],
            uv: Self::uv(&size.to_f32(), atlas_region),
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: texture.inner(),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: pixel_region.x as u32,
                    y: pixel_region.y as u32,
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
            texture,
            bgl,
            size,
            packer,
            images,
            white_pixel_handle: pixel_handle,
            handle: atlas_handle,
        }
    }

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

    fn write_region(&mut self, data: &[u8], size: Size<u32>) -> rect_packer::Rect {
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
        let region = self.write_region(&data, size);
        let handle = self.images.insert(Image {
            size,
            rgba: data,
            uv: Self::uv(&self.size.to_f32(), region),
        });

        info!(
            "Loaded image with size={}",
            ByteSize::from_bytes(bytes.len() as u64)
        );

        handle
    }

    pub fn register_font(&mut self, font: &mut Font) {
        for ch in font.chars() {
            let (size, bitmap) = font.rasterize_char(ch);

            if size.width == 0 || size.height == 0 {
                continue;
            }

            let mut rgba_data = Vec::with_capacity(bitmap.len() * 4);

            // Add a white character, will be recolored in the shaders
            for &alpha in &bitmap {
                rgba_data.extend_from_slice(&[255, 255, 255, alpha]);
            }

            let region = self.write_region(&rgba_data, size);
            let handle = self.images.insert(Image {
                size,
                rgba: rgba_data,
                uv: Self::uv(&self.size.to_f32(), region),
            });

            // Associate this character with its atlas image handle.
            font.insert_glyph_image(ch, size, handle);
        }
    }

    #[inline]
    pub fn get_image(&self, handle: Handle<Image>) -> &Image {
        self.images.get(handle).expect("Cannot find image")
    }

    #[inline]
    pub fn uv(atlas_size: &Size<f32>, region: rect_packer::Rect) -> Vector4 {
        Vector4::new(
            region.x as f32 / atlas_size.width,
            region.y as f32 / atlas_size.height,
            region.width as f32 / atlas_size.width,
            region.height as f32 / atlas_size.height,
        )
    }
}
