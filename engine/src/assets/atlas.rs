use std::sync::Arc;

use gpu::Gpu;
use gpu::Texture;
use gpu::TextureBatch;
use logging::fatal;
use utils::Handle;
use utils::PagePacker;

use crate::assets::image::DecodedImage;
use crate::assets::image::Image;

#[derive(Debug, Clone)]
pub struct PageData {
    pixels: Arc<Vec<u8>>,
    extent: u32,
    version: u64,
}

impl PageData {
    fn new(extent: u32) -> Self {
        Self {
            pixels: Arc::new(vec![0u8; (extent as usize).pow(2) * 4]),
            extent,
            version: 0,
        }
    }

    fn write(&mut self, data: &[u8], origin: math::Vector2<u32>, size: math::Size<u32>) {
        let pixels = Arc::make_mut(&mut self.pixels);
        let row_bytes = (size.width * 4) as usize;

        for row in 0..size.height {
            let src = (row * size.width * 4) as usize;
            let dst = (((origin.y + row) * TextureAtlas::PAGE_SIZE as u32 + origin.x) * 4) as usize;

            pixels[dst..dst + row_bytes].copy_from_slice(&data[src..src + row_bytes]);
        }

        self.version += 1;
    }

    fn extrude(&mut self, origin: math::Vector2<u32>, size: math::Size<u32>, pad: u32) {
        if pad == 0 || size.width == 0 || size.height == 0 {
            return;
        }

        let extent = self.extent;
        let (w, h) = (size.width, size.height);
        let (x0, y0) = (origin.x, origin.y);

        let left = pad.min(x0) as usize;
        let top = pad.min(y0) as usize;
        let right = pad.min(extent - (x0 + w)) as usize;
        let bottom = pad.min(extent - (y0 + h)) as usize;

        let stride = extent as usize * 4;
        let pixels = Arc::make_mut(&mut self.pixels);
        let (x0, y0, w, h) = (x0 as usize, y0 as usize, w as usize, h as usize);

        for row in y0..y0 + h {
            let base = row * stride;
            let l = base + x0 * 4;
            let r = base + (x0 + w - 1) * 4;

            let first: [u8; 4] = pixels[l..l + 4].try_into().unwrap();
            let last: [u8; 4] = pixels[r..r + 4].try_into().unwrap();

            for p in 1..=left {
                let o = base + (x0 - p) * 4;
                pixels[o..o + 4].copy_from_slice(&first);
            }

            for p in 1..=right {
                let o = base + (x0 + w - 1 + p) * 4;
                pixels[o..o + 4].copy_from_slice(&last);
            }
        }

        let span = (left + w + right) * 4;
        let x = (x0 - left) * 4;

        for p in 1..=top {
            let src = y0 * stride + x;
            pixels.copy_within(src..src + span, (y0 - p) * stride + x);
        }

        for p in 1..=bottom {
            let src = (y0 + h - 1) * stride + x;
            pixels.copy_within(src..src + span, (y0 + h - 1 + p) * stride + x);
        }
    }
}

pub enum PageKind {
    Shared,
    Dedicated(Handle<Image>),
}

pub struct Page {
    data: PageData,
    packer: PagePacker,
    kind: PageKind,
    texture: Option<gpu::Texture>,
    uploaded: u64,
}

impl Page {
    fn new(extent: u32, kind: PageKind) -> Self {
        Self {
            data: PageData::new(extent),
            packer: PagePacker::new(extent, TextureAtlas::PADDING),
            kind,
            texture: None,
            uploaded: 0,
        }
    }

    fn shared(extent: u32) -> Self {
        Self::new(extent, PageKind::Shared)
    }

    fn dedicated(extent: u32, owner: Handle<Image>) -> Self {
        Self::new(extent, PageKind::Dedicated(owner))
    }

    fn dirty(&self) -> bool {
        self.data.version != self.uploaded
    }
}

pub struct TextureAtlas {
    pages: Vec<Page>,
}

impl TextureAtlas {
    pub const PAGE_SIZE: u32 = 1024;
    pub const PADDING: u32 = 1;
    pub const SHARED_CAPACITY: u32 = Self::PAGE_SIZE - Self::PADDING * 2;

    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    /// The page's texture, or `None` when the page exists on the CPU but has
    /// not been uploaded yet.
    pub fn page_texture(&self, index: usize) -> Option<&Texture> {
        self.pages.get(index)?.texture.as_ref()
    }

    pub fn upload_dirty(&mut self, gpu: &Gpu) {
        let dirty = self
            .pages
            .iter()
            .enumerate()
            .filter(|(_, page)| page.dirty())
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        if dirty.is_empty() {
            return;
        }

        for &i in &dirty {
            let page = &mut self.pages[i];
            let extent = page.data.extent;

            page.texture.get_or_insert_with(|| {
                Texture::new(gpu, format!("atlas-page-{i}"), extent, extent)
            });
        }

        let bytes: usize = dirty
            .iter()
            .map(|&i| self.pages[i].data.pixels.len() + 256)
            .sum();

        let mut batch = TextureBatch::with_capacity(bytes);

        for &i in &dirty {
            let page = &self.pages[i];

            let Some(texture) = page.texture.as_ref() else {
                continue;
            };

            let extent = page.data.extent;
            batch.stage(texture, 0, 0, extent, extent, &page.data.pixels);
        }

        batch.submit(gpu);

        for &i in &dirty {
            let page = &mut self.pages[i];
            page.uploaded = page.data.version;
        }
    }

    fn place(
        page: &mut Page,
        index: usize,
        pixels: &[u8],
        size: math::Size<u32>,
        origin: math::Vector2<u32>,
    ) -> Image {
        page.data.write(pixels, origin, size);
        page.data.extrude(origin, size, TextureAtlas::PADDING);

        let extent = page.data.extent as f32;

        Image {
            page: index,
            uv_min: math::Vector2::new(origin.x as f32 / extent, origin.y as f32 / extent),
            uv_max: math::Vector2::new(
                (origin.x + size.width) as f32 / extent,
                (origin.y + size.height) as f32 / extent,
            ),
            size,
        }
    }

    pub fn insert(&mut self, dec: &DecodedImage, owner: Handle<Image>) -> Image {
        self.insert_rgba(&dec.pixels, dec.size, owner)
    }

    pub fn insert_rgba(
        &mut self,
        pixels: &[u8],
        size: math::Size<u32>,
        owner: Handle<Image>,
    ) -> Image {
        let expected = (size.width * size.height * 4) as usize;

        if pixels.len() != expected {
            // TODO: Placeholder
            fatal!(
                "Atlas insert: expected {} bytes for a {}x{} image, got {}",
                expected,
                size.width,
                size.height,
                pixels.len()
            );
        }

        if size.width > Self::SHARED_CAPACITY || size.height > Self::SHARED_CAPACITY {
            let extent = size.width.max(size.height) + Self::PADDING * 2;
            let index = self.pages.len();
            let mut page = Page::dedicated(extent, owner);

            let origin = page
                .packer
                .insert(size.width, size.height)
                .expect("a dedicated page always fits its own image");

            let origin = math::Vector2::new(origin.x, origin.y);
            let image = Self::place(&mut page, index, pixels, size, origin);
            self.pages.push(page);

            return image;
        }

        for index in 0..self.pages.len() {
            let page = &mut self.pages[index];

            if !matches!(page.kind, PageKind::Shared) {
                continue;
            }

            if let Some(origin) = page.packer.insert(size.width, size.height) {
                let origin = math::Vector2::new(origin.x, origin.y);
                return Self::place(page, index, pixels, size, origin);
            }
        }

        let index = self.pages.len();
        let mut page = Page::shared(Self::PAGE_SIZE);

        let origin = page
            .packer
            .insert(size.width, size.height)
            .expect("an empty shared page fits anything within SHARED_CAPACITY");

        let origin = math::Vector2::new(origin.x, origin.y);
        let image = Self::place(&mut page, index, pixels, size, origin);
        self.pages.push(page);

        image
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }
}
