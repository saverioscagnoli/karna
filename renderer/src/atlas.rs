use crate::mesh::material::Texture;
use image::GenericImageView;
use math::{Size, Vector2};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::BindGroupLayout;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl AtlasRegion {
    /// Converts atlas pixel coordinates to normalized UV coordinates (0.0-1.0)
    pub fn to_uv(&self, atlas_width: u32, atlas_height: u32) -> (Vector2, Vector2) {
        let uv_offset = Vector2::new(
            self.x as f32 / atlas_width as f32,
            self.y as f32 / atlas_height as f32,
        );

        let uv_scale = Vector2::new(
            self.width as f32 / atlas_width as f32,
            self.height as f32 / atlas_height as f32,
        );

        (uv_offset, uv_scale)
    }
}

#[derive(Debug)]
struct Shelf {
    y: u32,
    height: u32,
    x: u32, // Current x position in this shelf
}

#[derive(Debug)]
pub struct TextureAtlas {
    pub texture: Arc<Texture>,
    pub bind_group_layout: BindGroupLayout,
    pub size: Size<u32>,

    // Packing state
    shelves: Vec<Shelf>,
    current_y: u32,
    regions: HashMap<String, AtlasRegion>,
    padding: u32,
}

impl TextureAtlas {
    pub fn new(device: &wgpu::Device, size: Size<u32>) -> Self {
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

        let texture = Arc::new(Texture::create_empty(
            device,
            size,
            Some("Texture Atlas"),
            &bind_group_layout,
        ));

        Self {
            texture,
            bind_group_layout,
            size,
            shelves: Vec::new(),
            current_y: 0,
            regions: HashMap::new(),
            padding: 2, // 2 pixel padding to prevent bleeding
        }
    }

    /// Pack an image into the atlas using shelf packing algorithm
    pub fn pack(
        &mut self,
        queue: &wgpu::Queue,
        name: String,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<AtlasRegion, String> {
        // Check if already packed
        if let Some(region) = self.regions.get(&name) {
            return Ok(*region);
        }

        let padded_width = width + self.padding * 2;
        let padded_height = height + self.padding * 2;

        // Check if image fits in atlas
        if padded_width > self.size.width || padded_height > self.size.height {
            return Err(format!(
                "Image too large for atlas: {}x{} (atlas: {}x{})",
                width, height, self.size.width, self.size.height
            ));
        }

        // Try to find a shelf that can fit this image
        let mut region: Option<AtlasRegion> = None;

        for shelf in &mut self.shelves {
            // Check if image fits in this shelf
            if shelf.x + padded_width <= self.size.width && padded_height <= shelf.height {
                region = Some(AtlasRegion {
                    x: shelf.x + self.padding,
                    y: shelf.y + self.padding,
                    width,
                    height,
                });
                shelf.x += padded_width;
                break;
            }
        }

        // If no shelf found, create a new one
        if region.is_none() {
            if self.current_y + padded_height > self.size.height {
                return Err("Atlas is full".to_string());
            }

            let shelf = Shelf {
                y: self.current_y,
                height: padded_height,
                x: padded_width,
            };

            region = Some(AtlasRegion {
                x: self.padding,
                y: self.current_y + self.padding,
                width,
                height,
            });

            self.current_y += padded_height;
            self.shelves.push(shelf);
        }

        let region = region.unwrap();

        // Write the image data to the atlas
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x,
                    y: region.y,
                    z: 0,
                },
            },
            image_data,
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

        self.regions.insert(name, region);
        Ok(region)
    }

    /// Load an image from bytes and pack it into the atlas
    pub fn load_image(
        &mut self,
        queue: &wgpu::Queue,
        name: String,
        bytes: &[u8],
    ) -> Result<AtlasRegion, String> {
        let img =
            image::load_from_memory(bytes).map_err(|e| format!("Failed to load image: {}", e))?;

        let rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        self.pack(queue, name, &rgba, width, height)
    }

    /// Get a previously packed region by name
    pub fn get_region(&self, name: &str) -> Option<AtlasRegion> {
        self.regions.get(name).copied()
    }

    /// Get UV coordinates for a region by name
    pub fn get_uv(&self, name: &str) -> Option<(Vector2, Vector2)> {
        self.get_region(name)
            .map(|region| region.to_uv(self.size.width, self.size.height))
    }

    /// Get the bind group for rendering
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.texture.bind_group
    }

    /// Returns the percentage of atlas space used
    pub fn usage(&self) -> f32 {
        let total_pixels = self.size.width * self.size.height;
        let used_pixels: u32 = self
            .regions
            .values()
            .map(|r| (r.width + self.padding * 2) * (r.height + self.padding * 2))
            .sum();

        (used_pixels as f32 / total_pixels as f32) * 100.0
    }
}
