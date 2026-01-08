mod atlas;
mod font;

use atlas::TextureAtlas;
use globals::consts;
use logging::info;
use macros::Get;
use math::Size;
use std::path::Path;
use utils::{ByteSize, Handle, Label, SlotMap};

pub use font::*;

#[derive(Debug, Clone)]
pub struct Image {
    pub label: Label,
    pub size: Size<u32>,
}

#[derive(Get)]
pub struct AssetServer {
    atlas: TextureAtlas,

    fonts: SlotMap<Font>,

    #[get(copied)]
    debug_font: Handle<Font>,

    images: SlotMap<Image>,
}

impl AssetServer {
    #[doc(hidden)]
    pub fn new() -> Self {
        let atlas = TextureAtlas::new(consts::TEXTURE_ATLAS_BASE_SIZE);

        let mut assets = Self {
            atlas,
            fonts: SlotMap::new(),
            debug_font: Handle::default(),
            images: SlotMap::new(),
        };

        assets.init();
        assets
    }

    fn init(&mut self) {
        self.debug_font =
            self.load_font_bytes(include_bytes!("../defaults/DOS-V.ttf").to_vec(), 16);
    }

    /// Load an image from a path and return a handle
    pub fn load_image<P: AsRef<Path>>(&mut self, path: P) -> Handle<Image> {
        let bytes = std::fs::read(path).expect("Failed to find image");
        self.load_image_bytes(bytes)
    }

    /// Load an image from bytes and return a handle
    pub fn load_image_bytes(&mut self, bytes: Vec<u8>) -> Handle<Image> {
        let handle = self.images.insert_with_key(|key| {
            let label = Label::new(&format!("_img_{}", key.index()));

            info!(
                "Loading image with label {:?} of size {}",
                label,
                ByteSize::from_bytes(bytes.len() as u64)
            );

            let size = self.atlas.add_image_bytes(label, bytes);

            Image { label, size }
        });

        handle
    }

    pub fn load_font<P: AsRef<Path>>(&mut self, path: P, size: u8) -> Handle<Font> {
        let bytes = std::fs::read(path).expect("Failed to read font file");
        self.load_font_bytes(bytes, size)
    }

    pub fn load_font_bytes(&mut self, bytes: Vec<u8>, size: u8) -> Handle<Font> {
        info!(
            "Loading font of size {}",
            ByteSize::from_bytes(bytes.len() as u64)
        );

        let handle = self.fonts.insert_with_key(|key| {
            let label = Label::new(&format!("_font_{}", key.index()));
            let mut font = Font::new(label, bytes, size);

            self.atlas
                .rasterize_characters(label, &mut font, size as f32);

            font
        });

        handle
    }

    /// Get image metadata
    pub fn get_image(&self, handle: Handle<Image>) -> Option<&Image> {
        self.images.get(handle)
    }

    /// Get font
    pub fn get_font(&self, handle: Handle<Font>) -> Option<&Font> {
        self.fonts.get(handle)
    }

    // === Hidden Methods ===
    //
    // These methods are hidden since they must be used
    // in the renderer, but they shouldnt be exposed
    // to the user

    #[inline]
    #[doc(hidden)]
    pub fn get_texture_uv(&self, handle: Handle<Image>) -> (f32, f32, f32, f32, f32, f32) {
        let image = self.images.get(handle).expect("Invalid image handle");
        self.get_texture_uv_by_label(&image.label)
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_texture_uv_by_label(&self, label: &Label) -> (f32, f32, f32, f32, f32, f32) {
        let rect = self
            .atlas
            .regions
            .get(label)
            .expect("Failed to get atlas region");

        let size = self.atlas.size();
        let x = rect.x as f32 / size.width as f32;
        let y = rect.y as f32 / size.height as f32;
        let width = rect.width as f32 / size.width as f32;
        let height = rect.height as f32 / size.height as f32;

        (x, y, width, height, rect.width as f32, rect.height as f32)
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_white_uv_coords(&self) -> (f32, f32, f32, f32, f32, f32) {
        self.get_texture_uv_by_label(&utils::label!("_white"))
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_glyph_uv(&self, handle: Handle<Font>, ch: char) -> (f32, f32, f32, f32, f32, f32) {
        let font = self.fonts.get(handle).expect("Invalid font handle");
        let glyph_label = Label::new(&format!("{}_{}", font.label().raw(), ch));

        self.get_texture_uv_by_label(&glyph_label)
    }

    #[inline]
    #[doc(hidden)]
    pub fn atlas_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.atlas.bgl
    }

    #[inline]
    #[doc(hidden)]
    pub fn atlas_bg(&self) -> &wgpu::BindGroup {
        self.atlas.texture().bind_group()
    }
}
