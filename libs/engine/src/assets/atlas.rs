use nostd::alloc::format;
use nostd::alloc::vec::Vec;
use nostd::vec;
use sdl3::gpu::Device;
use sdl3::gpu::Texture;
use sdl3::gpu::TextureCopy;
use sdl3::gpu::TextureDesc;
use sdl3::gpu::TextureLocation;
use sdl3::gpu::TextureRegion;
use sdl3::gpu::TextureUsage;
use sdl3::image::DecodedImage;
use traccia::debug;
use traccia::error;

use crate::assets::image::Image;
use crate::assets::packer::Packer;
use crate::assets::packer::Placement;

const PADDING: u32 = 1;

pub struct TextureAtlas {
    device: Device,
    size: u32,
    texture: Texture,
    pages: Vec<Packer>,
    capacity: u32,
    max_pages: u32,
    generation: u32,
}

impl TextureAtlas {
    pub fn new(device: Device, size: u32, max_pages: u32) -> Self {
        assert!(max_pages >= 1, "atlas needs at least one page");

        let texture = Self::alloc_texture(&device, size, 1, 0);

        Self {
            device,
            size,
            texture,
            pages: vec![Packer::new(size, PADDING)],
            capacity: 1,
            max_pages,
            generation: 0,
        }
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn pages(&self) -> u32 {
        self.pages.len() as u32
    }
    pub fn generation(&self) -> u32 {
        self.generation
    }

    /// Packs `image` into the atlas and uploads its pixels.
    ///
    /// Returns `None` when the image is larger than a page, or when every page
    /// is full and no more can be opened; either way the caller has to fall
    /// back to a standalone texture.
    pub fn insert(&mut self, image: &DecodedImage) -> Option<Image> {
        let (w, h) = (image.width(), image.heigth());

        if w == 0 || h == 0 || w + PADDING > self.size || h + PADDING > self.size {
            return None;
        }

        let (page, placement) = self.alloc(w, h)?;

        let region = TextureRegion {
            mip_level: 0,
            layer: page,
            x: placement.x,
            y: placement.y,
            w,
            h,
        };

        if let Err(e) = self
            .device
            .upload_texture_region(&self.texture, region, image.pixels())
        {
            error!("Failed to upload image to atlas page {}: {}", page, e);
            return None;
        }

        Some(self.image(page, placement.x, placement.y, w, h))
    }

    fn alloc(&mut self, w: u32, h: u32) -> Option<(u32, Placement)> {
        for (i, packer) in self.pages.iter_mut().enumerate() {
            if let Some(placement) = packer.insert(w, h) {
                return Some((i as u32, placement));
            }
        }

        let next = self.pages.len() as u32;

        if next >= self.max_pages {
            return None;
        }

        if next == self.capacity && !self.grow() {
            return None;
        }

        let mut packer = Packer::new(self.size, PADDING);
        let placement = packer.insert(w, h)?;
        self.pages.push(packer);

        Some((next, placement))
    }

    fn grow(&mut self) -> bool {
        let capacity = self.capacity.saturating_mul(2).min(self.max_pages);

        if capacity <= self.capacity {
            return false;
        }

        let generation = self.generation + 1;
        let grown = Self::alloc_texture(&self.device, self.size, capacity, generation);

        let copies = (0..self.capacity)
            .map(|layer| TextureCopy {
                src: &self.texture,
                src_at: TextureLocation::layer(layer),
                dst: &grown,
                dst_at: TextureLocation::layer(layer),
                w: self.size,
                h: self.size,
            })
            .collect::<Vec<_>>();

        if let Err(e) = self.device.copy_textures(&copies) {
            error!("Failed to grow atlas to {} pages: {}", capacity, e);
            return false;
        }

        debug!("Atlas grown from {} to {} pages.", self.capacity, capacity);

        self.texture = grown;
        self.capacity = capacity;
        self.generation = generation;

        true
    }

    fn alloc_texture(device: &Device, size: u32, layers: u32, generation: u32) -> Texture {
        let desc = TextureDesc::rgba8_array(size, size, layers).with_usage(TextureUsage::SAMPLER);

        Texture::new(device.share(), format!("atlas#{}", generation), desc)
    }

    fn image(&self, page: u32, x: u32, y: u32, w: u32, h: u32) -> Image {
        // Half texel inset, so linear filtering never reaches into the gutter.
        let s = self.size as f32;

        Image::new(
            math::Size::new(w, h),
            page,
            math::vec2!((x as f32 + 0.5) / s, (y as f32 + 0.5) / s),
            math::vec2!(((x + w) as f32 - 0.5) / s, ((y + h) as f32 - 0.5) / s),
        )
    }
}
