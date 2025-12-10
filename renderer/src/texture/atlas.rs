use crate::{text::Font, texture::Texture};
use common::utils::Label;
use math::Size;
use std::sync::Arc;
use wgpu::naga::FastHashMap;

#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl AtlasRegion {
    /// Returns normalized UV coordinates (0.0 to 1.0) for this region
    pub fn uv_coords(&self, atlas_size: Size<u32>) -> UvCoords {
        UvCoords {
            min_x: self.x as f32 / atlas_size.width as f32,
            min_y: self.y as f32 / atlas_size.height as f32,
            max_x: (self.x + self.width) as f32 / atlas_size.width as f32,
            max_y: (self.y + self.height) as f32 / atlas_size.height as f32,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UvCoords {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

#[derive(Debug, Clone)]
pub struct TextureAtlas {
    pub texture: Arc<Texture>,
    pub bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub size: Size<u32>,
    regions: FastHashMap<Label, AtlasRegion>,
    /// Store original image data for repacking (using HashMap to avoid duplicates)
    images: FastHashMap<Label, image::RgbaImage>,
}

impl TextureAtlas {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, size: Size<u32>) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atlas texture bind group layout"),
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

        let bind_group_layout = Arc::new(bind_group_layout);

        let texture = Arc::new(Texture::new_empty(
            "Texture Atlas",
            device,
            size,
            &bind_group_layout,
        ));

        // Initialize the entire atlas with transparent pixels
        let clear_data = vec![0u8; (size.width * size.height * 4) as usize];

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.inner,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &clear_data,
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

        Self {
            texture,
            bind_group_layout,
            size,
            regions: FastHashMap::default(),
            images: FastHashMap::default(),
        }
    }

    #[inline]
    pub fn load_image(
        &mut self,
        queue: &wgpu::Queue,
        label: Label,
        bytes: &[u8],
    ) -> Result<AtlasRegion, String> {
        let img =
            image::load_from_memory(bytes).map_err(|e| format!("Failed to load image: {}", e))?;
        let rgba = img.to_rgba8();

        // Store or replace the image data (no duplicates)
        self.images.insert(label, rgba);

        // Repack all textures
        self.repack_all(queue)?;

        // Return the region for the newly added texture
        self.regions
            .get(&label)
            .copied()
            .ok_or_else(|| "Failed to find texture region after packing".to_string())
    }

    #[inline]
    pub fn load_font(
        &mut self,
        font: &Font,
        chars: &str,
        queue: &wgpu::Queue,
    ) -> Result<(), String> {
        for ch in chars.chars() {
            let (metrics, bitmap) = font.rasterize(ch);

            // Skip empty glyphs (like spaces)
            if metrics.width == 0 || metrics.height == 0 {
                continue;
            }

            // Convert grayscale bitmap to white RGBA texture
            let mut rgba_data = Vec::with_capacity(bitmap.len() * 4);
            for &alpha in &bitmap {
                rgba_data.push(255); // R - white
                rgba_data.push(255); // G - white
                rgba_data.push(255); // B - white
                rgba_data.push(alpha); // A - from glyph coverage
            }

            let rgba_image =
                image::RgbaImage::from_raw(metrics.width as u32, metrics.height as u32, rgba_data)
                    .ok_or_else(|| format!("Failed to create image for character '{}'", ch))?;

            let label = Label::new(&format!("{}_char_{}", font.label.raw(), ch as u32,));

            // Store the image in the atlas
            self.images.insert(label, rgba_image);
        }

        self.repack_all(queue)?;

        Ok(())
    }

    #[inline]
    fn repack_all(&mut self, queue: &wgpu::Queue) -> Result<(), String> {
        // Create items for all stored images
        let mut items = Vec::new();

        for (label, rgba) in &self.images {
            let (width, height) = rgba.dimensions();
            items.push(crunch::Item::new(
                *label,
                width as usize,
                height as usize,
                crunch::Rotation::None,
            ));
        }

        let container =
            crunch::Rect::new(0, 0, self.size.width as usize, self.size.height as usize);

        let packed = match crunch::pack(container, items) {
            Ok(all_packed) => all_packed,
            Err(_) => return Err("Failed to pack textures into atlas".to_string()),
        };

        self.regions.clear();

        for packed_item in &packed {
            let region = AtlasRegion {
                x: packed_item.rect.x as u32,
                y: packed_item.rect.y as u32,
                width: packed_item.rect.w as u32,
                height: packed_item.rect.h as u32,
            };

            // Find matching image data
            if let Some(rgba) = self.images.get(&packed_item.data) {
                self.write_texture_data(queue, rgba, region);
                self.regions.insert(packed_item.data, region);
            }
        }

        Ok(())
    }

    #[inline]
    fn write_texture_data(
        &self,
        queue: &wgpu::Queue,
        rgba: &image::RgbaImage,
        region: AtlasRegion,
    ) {
        let (img_width, img_height) = rgba.dimensions();

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture.inner,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x,
                    y: region.y,
                    z: 0,
                },
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * img_width),
                rows_per_image: Some(img_height),
            },
            wgpu::Extent3d {
                width: img_width,
                height: img_height,
                depth_or_array_layers: 1,
            },
        );
    }

    #[inline]
    pub(crate) fn get_uv_coords(&self, label: &Label) -> Option<UvCoords> {
        self.regions.get(label).map(|r| r.uv_coords(self.size))
    }

    #[inline]
    pub(crate) fn get_texture_size(&self, label: &Label) -> Option<Size<u32>> {
        self.regions
            .get(label)
            .map(|r| Size::new(r.width, r.height))
    }
}
