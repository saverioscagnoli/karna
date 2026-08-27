use std::rc::Rc;
use std::sync::Arc;

use logging::fatal;
use utils::Handle;
use utils::PagePacker;

use crate::assets::image::DecodedImage;
use crate::assets::image::Image;
use crate::config::config;
use crate::gpu::Device;
use crate::gpu::Filter;
use crate::gpu::Gpu;
use crate::gpu::Texture;
use crate::gpu::TextureDesc;

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
            let dst = (((origin.y + row) * self.extent + origin.x) * 4) as usize;

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

            let first: [u8; 4] = pixels[l..l + 4].try_into().expect("Failed to cast");
            let last: [u8; 4] = pixels[r..r + 4].try_into().expect("Failed to cast");

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

        self.version += 1;
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
    filter: Filter,
    texture: Option<Texture>,
    uploaded: u64,
}

impl Page {
    fn new(extent: u32, kind: PageKind, filter: Filter) -> Self {
        let config = config();

        Self {
            data: PageData::new(extent),
            packer: PagePacker::new(extent, config.asset.atlas_padding),
            kind,
            filter,
            texture: None,
            uploaded: 0,
        }
    }

    fn shared(extent: u32, filter: Filter) -> Self {
        Self::new(extent, PageKind::Shared, filter)
    }

    fn dedicated(extent: u32, owner: Handle<Image>, filter: Filter) -> Self {
        Self::new(extent, PageKind::Dedicated(owner), filter)
    }

    fn dirty(&self) -> bool {
        self.data.version != self.uploaded
    }
}

pub struct ImageView<'a> {
    pub pixels: &'a [u8],
    pub stride: usize,
    pub width: u32,
    pub height: u32,
}

impl<'a> ImageView<'a> {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    pub fn row(&self, y: u32) -> &'a [u8] {
        assert!(
            y < self.height,
            "row {y} is out of bounds for a {}px tall image",
            self.height
        );

        let start = y as usize * self.stride;

        &self.pixels[start..start + self.width as usize * 4]
    }

    pub fn rows(&self) -> impl Iterator<Item = &'a [u8]> + '_ {
        (0..self.height).map(|y| self.row(y))
    }

    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        assert!(
            x < self.width,
            "column {x} is out of bounds for a {}px wide image",
            self.width
        );

        let row = self.row(y);
        let o = x as usize * 4;

        [row[o], row[o + 1], row[o + 2], row[o + 3]]
    }

    pub fn to_rgba8(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.width as usize * self.height as usize * 4);

        for row in self.rows() {
            out.extend_from_slice(row);
        }

        out
    }
}

pub struct TextureAtlas {
    pages: Vec<Page>,
}

impl TextureAtlas {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    fn padding() -> u32 {
        config().asset.atlas_padding
    }

    fn page_size() -> u32 {
        config().asset.atlas_page_size
    }

    fn shared_capacity() -> u32 {
        Self::page_size() - Self::padding() * 2
    }

    pub fn page_texture(&self, index: usize) -> Option<&Texture> {
        self.pages.get(index)?.texture.as_ref()
    }

    pub fn page_filter(&self, index: usize) -> Option<Filter> {
        Some(self.pages.get(index)?.filter)
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn view(&self, image: &Image) -> ImageView<'_> {
        let Some(page) = self.pages.get(image.page) else {
            fatal!("Image references a missing atlas page: {}", image.page);
        };

        let extent = page.data.extent as usize;
        let stride = extent * 4;

        if image.size.width == 0 || image.size.height == 0 {
            return ImageView {
                pixels: &[],
                stride,
                width: 0,
                height: 0,
            };
        }

        let start = (image.origin.y as usize * extent + image.origin.x as usize) * 4;
        let len = (image.size.height as usize - 1) * stride + image.size.width as usize * 4;

        ImageView {
            pixels: &page.data.pixels[start..start + len],
            stride,
            width: image.size.width,
            height: image.size.height,
        }
    }

    pub fn upload_dirty(&mut self, device: &Device) {
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
                Texture::new(
                    device.clone(),
                    format!("atlas-page-{i}"),
                    TextureDesc::rgba8(extent, extent),
                )
            });
        }

        let uploads = dirty
            .iter()
            .filter_map(|&i| {
                let page = &self.pages[i];
                let texture = page.texture.as_ref()?;

                Some((texture, page.data.pixels.as_slice()))
            })
            .collect::<Vec<_>>();

        if let Err(err) = device.upload_textures(&uploads) {
            fatal!("Atlas upload failed: {:?}", err);
        }

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
        page.data.extrude(origin, size, Self::padding());

        let extent = page.data.extent as f32;

        Image {
            page: index,
            origin,
            uv_min: math::Vector2::new(origin.x as f32 / extent, origin.y as f32 / extent),
            uv_max: math::Vector2::new(
                (origin.x + size.width) as f32 / extent,
                (origin.y + size.height) as f32 / extent,
            ),
            size,
        }
    }

    pub fn insert(&mut self, dec: &DecodedImage, owner: Handle<Image>, filter: Filter) -> Image {
        self.insert_rgba(&dec.rgba, dec.size, owner, filter)
    }

    pub fn insert_rgba(
        &mut self,
        pixels: &[u8],
        size: math::Size<u32>,
        owner: Handle<Image>,
        filter: Filter,
    ) -> Image {
        let expected = (size.width * size.height * 4) as usize;

        if pixels.len() != expected {
            fatal!(
                "Atlas insert: expected {} bytes for a {}x{} image, got {}",
                expected,
                size.width,
                size.height,
                pixels.len()
            );
        }

        let capacity = Self::shared_capacity();

        if size.width > capacity || size.height > capacity {
            let extent = size.width.max(size.height) + Self::padding() * 2;
            let index = self.pages.len();
            let mut page = Page::dedicated(extent, owner, filter);

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

            if !matches!(page.kind, PageKind::Shared) || page.filter != filter {
                continue;
            }

            if let Some(origin) = page.packer.insert(size.width, size.height) {
                let origin = math::Vector2::new(origin.x, origin.y);
                return Self::place(page, index, pixels, size, origin);
            }
        }

        let index = self.pages.len();
        let mut page = Page::shared(Self::page_size(), filter);

        let origin = page
            .packer
            .insert(size.width, size.height)
            .expect("an empty shared page fits anything within shared_capacity");

        let origin = math::Vector2::new(origin.x, origin.y);
        let image = Self::place(&mut page, index, pixels, size, origin);
        self.pages.push(page);

        image
    }
}
