use macros::Get;
use math::Size;
use utils::{FastHashMap, Label, label};

use crate::font::Font;

#[derive(Get)]
pub struct TextureAtlas {
    #[get]
    texture: gpu::Texture,

    #[get]
    pub bgl: wgpu::BindGroupLayout,

    #[get]
    size: Size<u32>,

    packer: rect_packer::DensePacker,
    pub regions: FastHashMap<Label, rect_packer::Rect>,
}

impl TextureAtlas {
    #[doc(hidden)]
    pub fn new<S>(size: S) -> Self
    where
        S: Into<Size<u32>>,
    {
        let size: Size<u32> = size.into();
        let device = gpu::device();

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

        let queue = gpu::queue();
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

        regions.insert(label!("_white"), white_pixel);

        regions.insert(
            label!("_atlas"),
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

    fn write_rgba(&self, rgba: &[u8], width: u32, height: u32, region: &rect_packer::Rect) {
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
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Add raw RGBA image data to the atlas
    pub fn add_rgba(&mut self, label: Label, rgba: &[u8], width: u32, height: u32) -> Size<u32> {
        debug_assert_eq!(
            rgba.len(),
            (width * height * 4) as usize,
            "RGBA buffer size mismatch"
        );

        let region = self
            .packer
            .pack(width as i32, height as i32, false)
            .expect("Failed to pack image into atlas");

        self.write_rgba(rgba, width, height, &region);
        self.regions.insert(label, region);

        Size::new(width, height)
    }

    pub fn rasterize_characters(&mut self, label: Label, font: &mut Font, size: f32) {
        let chars = font.chars().keys().copied().collect::<Vec<_>>();

        for ch in chars {
            let (metrics, bitmap) = font.rasterize(ch, size);
            let width = metrics.width as u32;
            let height = metrics.height as u32;

            if width == 0 || height == 0 {
                continue;
            }

            font.add_glyph(ch, width, height);

            // Convert grayscale alpha to white RGBA
            let mut rgba = Vec::with_capacity(bitmap.len() * 4);
            for &alpha in &bitmap {
                rgba.extend_from_slice(&[255, 255, 255, alpha]);
            }

            let glyph_label = Label::new(&format!("{}_{}", label.raw(), ch));
            self.add_rgba(glyph_label, &rgba, width, height);
        }
    }
}
