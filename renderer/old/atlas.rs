use math::{Size, Vec2};
use std::collections::HashMap;
use traccia::{error, info};

#[derive(Debug, Clone)]
pub struct AtlasRegion {
    pub size: Size<u32>,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
}

pub struct TextureAtlas {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: Size<u32>,
    regions: HashMap<String, AtlasRegion>,
    pub fonts: HashMap<String, fontdue::Font>,
    char_regions: HashMap<char, AtlasRegion>,
    free_rectangles: Vec<AtlasRect>,
}

#[derive(Debug, Clone)]
struct AtlasRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl AtlasRect {
    fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn can_fit(&self, width: u32, height: u32) -> bool {
        self.width >= width && self.height >= height
    }

    fn split(&self, width: u32, height: u32) -> Vec<AtlasRect> {
        let mut result = Vec::new();

        // Right rectangle
        if self.width > width {
            result.push(AtlasRect::new(
                self.x + width,
                self.y,
                self.width - width,
                height,
            ));
        }

        // Bottom rectangle
        if self.height > height {
            result.push(AtlasRect::new(
                self.x,
                self.y + height,
                self.width,
                self.height - height,
            ));
        }

        // Bottom-right rectangle (if both dimensions are larger)
        if self.width > width && self.height > height {
            result.push(AtlasRect::new(
                self.x + width,
                self.y + height,
                self.width - width,
                self.height - height,
            ));
        }

        result
    }
}

impl TextureAtlas {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, initial_size: u32) -> Self {
        let size = Size::new(initial_size, initial_size);
        let (texture, view, sampler) = Self::create_white_texture(device, queue, size);

        let mut free_rectangles = Vec::new();
        free_rectangles.push(AtlasRect::new(0, 0, initial_size, initial_size));

        Self {
            texture,
            view,
            sampler,
            size,
            regions: HashMap::new(),
            char_regions: HashMap::new(),
            fonts: HashMap::new(),
            free_rectangles,
        }
    }

    fn create_white_texture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: Size<u32>,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler) {
        let texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Atlas Texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // Fill with white pixels
        let white_data = vec![255u8; (size.width * size.height * 4) as usize];
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &white_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.width),
                rows_per_image: Some(size.height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        (texture, view, sampler)
    }

    pub fn load_font(&mut self, label: &str, bytes: &[u8]) {
        if let Ok(font) = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()) {
            self.fonts.insert(label.to_string(), font);
        } else {
            error!("Failed to load font from bytes");
        }
    }

    pub fn draw_text(&mut self, font: &str, text: &str) {}

    // Updated method to handle text and add characters to atlas
    pub fn handle_text(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font: &str,
        text: &str,
        font_size: f32,
    ) {
        let font = match self.fonts.get(font) {
            Some(f) => f.clone(),
            None => {
                error!("Font '{}' not found in atlas", font);
                return;
            }
        };

        for c in text.chars() {
            if self.char_regions.contains_key(&c) || c.is_whitespace() {
                // Character already exists in atlas, no need to add
                continue;
            }

            let (metrics, coverage) = font.rasterize(c, font_size);

            // Skip characters with no pixels (like spaces)
            if coverage.is_empty() || metrics.width == 0 || metrics.height == 0 {
                continue;
            }

            // Convert grayscale coverage to white RGBA texture data
            let char_data = self.coverage_to_white_rgba(&coverage);
            let char_size = Size::new(metrics.width as u32, metrics.height as u32);

            // Add the character texture to the atlas
            if self.add_char_texture(device, queue, c, &char_data, char_size) {
                info!("Added character '{}' to atlas", c);
            } else {
                error!("Failed to add character '{}' to atlas", c);
            }
        }
    }

    // Convert grayscale coverage data to white RGBA data
    fn coverage_to_white_rgba(&self, coverage: &[u8]) -> Vec<u8> {
        let mut rgba_data = Vec::with_capacity(coverage.len() * 4);

        for &alpha in coverage {
            // White color with varying alpha based on coverage
            rgba_data.push(255); // R
            rgba_data.push(255); // G
            rgba_data.push(255); // B
            rgba_data.push(alpha); // A
        }

        rgba_data
    }

    // Add a character texture to the atlas
    fn add_char_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        character: char,
        char_data: &[u8],
        char_size: Size<u32>,
    ) -> bool {
        // Try to find a free rectangle that can fit the character
        let mut best_rect_index = None;
        let mut best_area = u32::MAX;

        for (i, rect) in self.free_rectangles.iter().enumerate() {
            if rect.can_fit(char_size.width, char_size.height) {
                let area = rect.width * rect.height;
                if area < best_area {
                    best_area = area;
                    best_rect_index = Some(i);
                }
            }
        }

        if let Some(rect_index) = best_rect_index {
            let rect = self.free_rectangles.remove(rect_index);

            // Create the region
            let region = AtlasRegion {
                size: char_size,
                uv_min: Vec2::new(
                    rect.x as f32 / self.size.width as f32,
                    rect.y as f32 / self.size.height as f32,
                ),
                uv_max: Vec2::new(
                    (rect.x + char_size.width) as f32 / self.size.width as f32,
                    (rect.y + char_size.height) as f32 / self.size.height as f32,
                ),
            };

            // Split the rectangle and add remaining parts back to free list
            let split_rects = rect.split(char_size.width, char_size.height);
            self.free_rectangles.extend(split_rects);

            // Write the character texture data to the atlas
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: rect.x,
                        y: rect.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                char_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * char_size.width),
                    rows_per_image: Some(char_size.height),
                },
                wgpu::Extent3d {
                    width: char_size.width,
                    height: char_size.height,
                    depth_or_array_layers: 1,
                },
            );

            self.char_regions.insert(character, region);
            true
        } else {
            // Need to expand the atlas
            self.expand_atlas(device, queue);
            // Try again after expansion
            self.add_char_texture(device, queue, character, char_data, char_size)
        }
    }

    pub fn add_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        name: &str,
        image_data: &[u8],
        image_size: Size<u32>,
        resized: Option<bool>,
    ) -> bool {
        // Try to find a free rectangle that can fit the texture
        let mut best_rect_index = None;
        let mut best_area = u32::MAX;

        for (i, rect) in self.free_rectangles.iter().enumerate() {
            if rect.can_fit(image_size.width, image_size.height) {
                let area = rect.width * rect.height;
                if area < best_area {
                    best_area = area;
                    best_rect_index = Some(i);
                }
            }
        }

        if let Some(rect_index) = best_rect_index {
            let rect = self.free_rectangles.remove(rect_index);

            // Create the region
            let region = AtlasRegion {
                size: image_size,
                uv_min: Vec2::new(
                    rect.x as f32 / self.size.width as f32,
                    rect.y as f32 / self.size.height as f32,
                ),
                uv_max: Vec2::new(
                    (rect.x + image_size.width) as f32 / self.size.width as f32,
                    (rect.y + image_size.height) as f32 / self.size.height as f32,
                ),
            };

            // Split the rectangle and add remaining parts back to free list
            let split_rects = rect.split(image_size.width, image_size.height);
            self.free_rectangles.extend(split_rects);

            // Write the texture data to the atlas
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: rect.x,
                        y: rect.y,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                image_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * image_size.width),
                    rows_per_image: Some(image_size.height),
                },
                wgpu::Extent3d {
                    width: image_size.width,
                    height: image_size.height,
                    depth_or_array_layers: 1,
                },
            );

            self.regions.insert(name.to_string(), region.clone());
            resized.unwrap_or(false)
        } else {
            // Need to expand the atlas
            self.expand_atlas(device, queue);
            // Try again after expansion
            self.add_texture(device, queue, name, image_data, image_size, Some(true))
        }
    }

    fn expand_atlas(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let old_size = self.size; // Store the old size before updating
        let new_size = Size::new(self.size.width * 2, self.size.height * 2);

        // Create new larger texture
        let (new_texture, new_view, new_sampler) =
            Self::create_white_texture(device, queue, new_size);

        // Copy old texture data to new texture
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Atlas Expansion Encoder"),
        });

        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyTextureInfo {
                texture: &new_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::Extent3d {
                width: self.size.width,
                height: self.size.height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit([encoder.finish()]);

        // Update atlas
        self.texture = new_texture;
        self.view = new_view;
        self.sampler = new_sampler;

        // Update UV coordinates for existing regions BEFORE updating self.size
        for region in self.regions.values_mut() {
            // Convert back to pixel coordinates using the OLD atlas size
            let pixel_min_x = (region.uv_min.x * old_size.width as f32) as u32;
            let pixel_min_y = (region.uv_min.y * old_size.height as f32) as u32;
            let pixel_max_x = (region.uv_max.x * old_size.width as f32) as u32;
            let pixel_max_y = (region.uv_max.y * old_size.height as f32) as u32;

            // Recalculate UV coordinates with new atlas size
            region.uv_min.x = pixel_min_x as f32 / new_size.width as f32;
            region.uv_min.y = pixel_min_y as f32 / new_size.height as f32;
            region.uv_max.x = pixel_max_x as f32 / new_size.width as f32;
            region.uv_max.y = pixel_max_y as f32 / new_size.height as f32;
        }

        // Update UV coordinates for character regions too
        for region in self.char_regions.values_mut() {
            // Convert back to pixel coordinates using the OLD atlas size
            let pixel_min_x = (region.uv_min.x * old_size.width as f32) as u32;
            let pixel_min_y = (region.uv_min.y * old_size.height as f32) as u32;
            let pixel_max_x = (region.uv_max.x * old_size.width as f32) as u32;
            let pixel_max_y = (region.uv_max.y * old_size.height as f32) as u32;

            // Recalculate UV coordinates with new atlas size
            region.uv_min.x = pixel_min_x as f32 / new_size.width as f32;
            region.uv_min.y = pixel_min_y as f32 / new_size.height as f32;
            region.uv_max.x = pixel_max_x as f32 / new_size.width as f32;
            region.uv_max.y = pixel_max_y as f32 / new_size.height as f32;
        }

        // Add new free rectangles for the expanded areas
        // Right side of the expanded texture
        self.free_rectangles.push(AtlasRect::new(
            old_size.width,
            0,
            old_size.width,
            old_size.height,
        ));
        // Bottom side of the expanded texture
        self.free_rectangles.push(AtlasRect::new(
            0,
            old_size.height,
            old_size.width,
            old_size.height,
        ));
        // Bottom-right corner
        self.free_rectangles.push(AtlasRect::new(
            old_size.width,
            old_size.height,
            old_size.width,
            old_size.height,
        ));

        // Update size AFTER recalculating UVs
        self.size = new_size;

        info!(
            "Expanded texture atlas to {}x{}",
            self.size.width, self.size.height
        );
    }

    pub fn create_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    pub fn get_region(&self, name: &str) -> Option<&AtlasRegion> {
        self.regions.get(name)
    }

    // Get a character region from the atlas
    pub fn get_char_region(&self, character: char) -> Option<&AtlasRegion> {
        self.char_regions.get(&character)
    }
}
