use packer::PagePacker;
use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;
use utils::hash_bytes;

use crate::render::layouts;

const PAGE_SIZE: u32 = 1024;
const PADDING: u32 = 1;

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct Image {
    pub page: u32,
    pub uv_min: math::Vector2<f32>,
    pub uv_max: math::Vector2<f32>,
    pub size: math::Size<u32>,
}

#[derive(Debug, Clone)]
struct Page {
    texture: gpu::Texture,
    packer: PagePacker,
    dedicated: bool,
    handle: Handle<Image>,
}

#[derive(Debug, Clone)]
pub struct TextureAtlas {
    pages: Vec<Page>,
    images: SlotMap<Image>,
    by_content: FastHashMap<u64, Handle<Image>>,
    counter: u64,

    white_pixel: Image,
}

impl TextureAtlas {
    pub fn new() -> Self {
        let mut atlas = Self {
            pages: Vec::new(),
            images: SlotMap::new(),
            by_content: FastHashMap::default(),
            counter: 0,
            white_pixel: Image::default(),
        };

        const WHITE: [u8; 4 * 4 * 4] = [0xFF; 64];
        let h = atlas.insert(&WHITE, math::Size::new(4, 4));
        let white_pixel = atlas.get(h).clone();

        atlas.white_pixel = white_pixel;

        atlas
    }

    fn place(
        page: &mut Page,
        page_idx: u32,
        pos: packer::Placement,
        data: &[u8],
        size: math::Size<u32>,
    ) -> Image {
        page.texture
            .write(data, math::Vector2::new(pos.x, pos.y), size);

        let inv = 1.0 / PAGE_SIZE as f32;

        Image {
            page: page_idx,
            uv_min: math::Vector2::new(pos.x as f32 * inv, pos.y as f32 * inv),
            uv_max: math::Vector2::new(
                (pos.x + size.width) as f32 * inv,
                (pos.y + size.height) as f32 * inv,
            ),
            size,
        }
    }

    fn new_page(&mut self) -> usize {
        let page_idx = self.pages.len() as u32;
        let plabel = format!("texture n.{}-page{}", self.counter, page_idx);
        self.counter += 1;

        let texture = gpu::Texture::new_blank(
            plabel,
            math::Size::new(PAGE_SIZE, PAGE_SIZE),
            layouts::atlas(),
        );

        // A phantom image covering the whole page, for debug display.
        let handle = self.images.insert(Image {
            page: page_idx,
            uv_min: math::Vector2::new(0.0, 0.0),
            uv_max: math::Vector2::new(1.0, 1.0),
            size: math::Size::new(PAGE_SIZE, PAGE_SIZE),
        });

        self.pages.push(Page {
            texture,
            packer: PagePacker::new(PAGE_SIZE, PADDING),
            dedicated: false,
            handle,
        });

        self.pages.len() - 1
    }

    fn alloc_packed(&mut self, data: &[u8], size: math::Size<u32>) -> Image {
        for (idx, page) in self.pages.iter_mut().enumerate() {
            if !page.dedicated {
                if let Some(pos) = page.packer.insert(size.width, size.height) {
                    return Self::place(page, idx as u32, pos, data, size);
                }
            }
        }

        let idx = self.new_page();
        let pos = self.pages[idx]
            .packer
            .insert(size.width, size.height)
            .expect("under-threshold image must fit an empty page");

        Self::place(&mut self.pages[idx], idx as u32, pos, data, size)
    }

    pub fn insert(&mut self, data: &[u8], size: math::Size<u32>) -> Handle<Image> {
        let hash = hash_bytes(data);

        if let Some(&h) = self.by_content.get(&hash) {
            return h;
        }

        let image = self.alloc_packed(data, size);
        let handle = self.images.insert(image);

        self.by_content.insert(hash, handle);

        handle
    }

    pub fn get(&self, h: Handle<Image>) -> &Image {
        &self.images[h]
    }

    pub fn page_bind_group(&self, index: usize) -> &gpu::BindGroup {
        &self.pages[index].texture.bind_group
    }

    pub fn all_bind_groups(&self) -> impl Iterator<Item = (u32, &gpu::BindGroup)> + '_ {
        self.pages
            .iter()
            .enumerate()
            .map(|(i, p)| (i as u32, &p.texture.bind_group))
    }

    pub fn all_handles(&self) -> impl Iterator<Item = Handle<Image>> + '_ {
        self.pages.iter().map(|p| p.handle)
    }

    pub fn white_pixel(&self) -> &Image {
        &self.white_pixel
    }
}
