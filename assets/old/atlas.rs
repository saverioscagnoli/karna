use gpu::Texture;
use image::{DynamicImage, GenericImageView, RgbaImage};
use macros::Get;
use math::Size;
use rect_packer::Packer;
use std::sync::{Mutex, RwLock};
use utils::{Label, LabelMap, label};

use crate::font::Font;

#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl From<rect_packer::Rect> for AtlasRegion {
    fn from(value: rect_packer::Rect) -> Self {
        Self {
            x: value.x as u32,
            y: value.y as u32,
            width: value.width as u32,
            height: value.height as u32,
        }
    }
}

impl AtlasRegion {
    #[inline]
    /// (x, y, width, height)
    pub fn uv_coordinates(&self, atlas_size: &Size<u32>) -> (f32, f32, f32, f32) {
        let x = self.x as f32 / atlas_size.width() as f32;
        let y = self.y as f32 / atlas_size.height() as f32;
        let width = self.width as f32 / atlas_size.width() as f32;
        let height = self.height as f32 / atlas_size.height() as f32;

        (x, y, width, height)
    }
}

#[derive(Get)]
pub struct TextureAtlas {
    #[get]
    texture: Texture,

    #[get]
    bind_group_layout: wgpu::BindGroupLayout,

    #[get]
    size: Size<u32>,

    /// Can be a mutex, since it's mainly used for writing (lock, but it's ok, loading images takes time),
    /// when getting a texture we don't need to lock it
    packer: Mutex<rect_packer::Packer>,
    regions: RwLock<LabelMap<AtlasRegion>>,
}

impl TextureAtlas {
    pub fn new<S: Into<Size<u32>>>(size: S) -> Self {
        let size = size.into();
        let device = gpu::device();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("texture atlas bgl"),
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

        let texture = Texture::new_empty("texture atlas", size, &bind_group_layout, device);

        let mut packer = Packer::new(rect_packer::Config {
            width: size.width() as i32,
            height: size.height() as i32,
            border_padding: 0,
            rectangle_padding: 0,
        });

        // Reserve a 1x1 region for white pixel
        let white_region: AtlasRegion = packer
            .pack(1, 1, false)
            .expect("Failed to pack white pixel")
            .into();

        // Write white pixel to the atlas
        gpu::queue().write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: texture.inner(),
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: white_region.x,
                    y: white_region.y,
                    z: 0,
                },
            },
            &[255u8, 255u8, 255u8, 255u8], // White RGBA pixel
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

        let mut regions = LabelMap::default();

        // Store the white pixel region
        regions.insert(label!("_white"), white_region);

        // Store the entire atlas region for debugging
        regions.insert(
            label!("_atlas"),
            AtlasRegion {
                x: 0,
                y: 0,
                width: size.width(),
                height: size.height(),
            },
        );

        Self {
            texture,
            bind_group_layout,
            size,
            packer: Mutex::new(packer),
            regions: RwLock::new(regions),
        }
    }

    fn write_texture(&self, image: DynamicImage, size: Size<u32>, region: AtlasRegion) {
        gpu::queue().write_texture(
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
            image.to_rgba8().as_raw(),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width()),
                rows_per_image: Some(size.height()),
            },
            wgpu::Extent3d {
                width: size.width(),
                height: size.height(),
                depth_or_array_layers: 1,
            },
        );
    }

    pub fn get_region(&self, label: Label) -> AtlasRegion {
        *self
            .regions
            .read()
            .expect("Texture atlas lock is poisoned")
            .get(&label)
            .expect("Failed to get region")
    }

    #[inline]
    /// Returns UV coordinates for the white pixel in the atlas
    pub fn get_white_uv_coords(&self) -> (f32, f32, f32, f32) {
        self.get_region(label!("_white")).uv_coordinates(&self.size)
    }

    #[inline]
    pub fn add_image(&self, label: Label, bytes: Vec<u8>) {
        let image = image::load_from_memory(&bytes).expect("Failed to load image");
        let (width, height) = image.dimensions();
        let size = Size::new(width, height);
        let mut packer = self.packer.lock().expect("Failed to write to packer");

        let region: AtlasRegion = packer
            .pack(size.width() as i32, size.height() as i32, false)
            .expect("Failed to pack image")
            .into();

        self.write_texture(image, size, region);

        let mut regions = self
            .regions
            .write()
            .expect("Texture atlas lock is poisoned");

        regions.insert(label, region);
    }

    #[inline]
    /// Helper method to load characters textures.
    ///
    /// Cannot use `add_image` before its meant for the end user to load
    /// images when reading files
    fn add_raw_image(&self, label: Label, image: RgbaImage) {
        let (width, height) = image.dimensions();
        let size = Size::new(width, height);
        let mut packer = self.packer.lock().expect("Failed to write to packer");
        let region: AtlasRegion = packer
            .pack(size.width() as i32, size.height() as i32, false)
            .expect("Failed to pack image")
            .into();

        self.write_texture(DynamicImage::ImageRgba8(image), size, region);

        let mut regions = self
            .regions
            .write()
            .expect("Texture atlas lock is poisoned");

        regions.insert(label, region);
    }

    #[inline]
    pub fn rasterize_characters(&self, font_label: Label, font: &mut Font, size: f32) {
        let chars = font.chars().keys().copied().collect::<Vec<_>>();

        for ch in chars {
            let (metrics, bitmap) = font.rasterize(ch, size);
            let (width, height) = (metrics.width as u32, metrics.height as u32);

            if width == 0 || height == 0 {
                continue;
            }

            font.add_glyph(ch, width, height);

            // Just load a white sample of the character,
            // Keeping only the alpha values that define the character.
            // Color and transform can be handled via transforms
            // and material changes
            let mut rgba_buffer = Vec::with_capacity(bitmap.len() * 4);

            for &alpha in &bitmap {
                rgba_buffer.extend_from_slice(&[255, 255, 255, alpha]);
            }

            let texture = RgbaImage::from_raw(width, height, rgba_buffer)
                .expect("Failed to create char texture");

            // Store the character with label {font_label}_{ch}
            let label = Label::new(&format!("{}_{}", font_label.raw(), ch));

            self.add_raw_image(label, texture);
        }
    }
}
