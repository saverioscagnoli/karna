mod atlas;
mod font;

use atlas::TextureAtlas;
use globals::consts;
use logging::info;
use macros::Get;
use math::Size;
use parking_lot::{MappedRwLockReadGuard, RwLock, RwLockReadGuard};
use std::{io::Cursor, path::Path, sync::Arc};
use utils::{ByteSize, Handle, Label, SlotMap};

pub use font::*;

#[derive(Debug, Clone)]
pub struct Image {
    pub label: Label,
    pub size: Size<u32>,
}

#[derive(Clone)]
#[derive(Get)]
pub struct AssetServer {
    atlas: Arc<RwLock<TextureAtlas>>,
    images: Arc<RwLock<SlotMap<Image>>>,
    fonts: Arc<RwLock<SlotMap<Font>>>,

    #[get(copied)]
    debug_font: Handle<Font>,
}

impl AssetServer {
    #[doc(hidden)]
    pub fn new() -> Self {
        let atlas = TextureAtlas::new(consts::TEXTURE_ATLAS_BASE_SIZE);

        let mut server = Self {
            atlas: Arc::new(RwLock::new(atlas)),
            images: Arc::new(RwLock::new(SlotMap::new())),
            fonts: Arc::new(RwLock::new(SlotMap::new())),
            debug_font: Handle::default(),
        };

        server.init();
        server
    }

    fn init(&mut self) {
        self.debug_font =
            self.load_font_bytes(include_bytes!("../defaults/DOS-V.ttf").to_vec(), 16);
    }

    #[inline]
    #[doc(hidden)]
    pub fn guard(&self) -> AssetServerGuard<'_> {
        AssetServerGuard {
            atlas: self.atlas.read(),
            images: self.images.read(),
            fonts: self.fonts.read(),
            debug_font: self.debug_font,
        }
    }

    pub fn load_image_bytes(&self, bytes: Vec<u8>) -> Handle<Image> {
        let (rgba, width, height) = Self::decode_png(&bytes);

        let mut images = self.images.write();
        let mut atlas = self.atlas.write();

        images.insert_with_key(|key| {
            info!(
                "Loading image {}x{} ({})",
                width,
                height,
                ByteSize::from_bytes(rgba.len() as u64)
            );
            let label = Label::new(&format!("_img_{}", key.index()));
            let size = atlas.add_rgba(label, &rgba, width, height);

            Image { label, size }
        })
    }

    fn decode_png(bytes: &[u8]) -> (Vec<u8>, u32, u32) {
        let mut decoder = png::Decoder::new(Cursor::new(bytes));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::ALPHA);

        let mut reader = decoder.read_info().expect("Failed to read PNG info");
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buf).expect("Failed to decode PNG");
        buf.truncate(info.buffer_size());

        (buf, info.width, info.height)
    }

    pub fn load_image<P: AsRef<Path>>(&self, path: P) -> Handle<Image> {
        let bytes = std::fs::read(path).expect("Failed to read image file");
        self.load_image_bytes(bytes)
    }

    pub fn load_font_bytes(&self, bytes: Vec<u8>, size: u8) -> Handle<Font> {
        let mut fonts = self.fonts.write();
        let mut atlas = self.atlas.write();

        fonts.insert_with_key(|key| {
            info!(
                "Loading font of size {}",
                ByteSize::from_bytes(bytes.len() as u64)
            );
            let label = Label::new(&format!("_font_{}", key.index()));
            let mut font = Font::new(label, bytes, size);

            atlas.rasterize_characters(label, &mut font, size as f32);
            font
        })
    }

    pub fn load_font<P: AsRef<Path>>(&self, path: P, size: u8) -> Handle<Font> {
        let bytes = std::fs::read(path).expect("Failed to read font file");
        self.load_font_bytes(bytes, size)
    }

    #[inline]
    pub fn get_image(&self, handle: Handle<Image>) -> MappedRwLockReadGuard<'_, Image> {
        let guard = self.images.read();

        RwLockReadGuard::map(guard, |images| images.get(handle).expect("Image not found"))
    }

    #[inline]
    pub fn get_font(&self, handle: Handle<Font>) -> MappedRwLockReadGuard<'_, Font> {
        let guard = self.fonts.read();

        RwLockReadGuard::map(guard, |fonts| fonts.get(handle).expect("Font not found"))
    }
}

#[derive(Get)]
pub struct AssetServerGuard<'a> {
    atlas: RwLockReadGuard<'a, TextureAtlas>,
    images: RwLockReadGuard<'a, SlotMap<Image>>,
    fonts: RwLockReadGuard<'a, SlotMap<Font>>,

    #[get(copied)]
    debug_font: Handle<Font>,
}

impl<'a> AssetServerGuard<'a> {
    #[inline]
    pub fn get_image(&self, handle: Handle<Image>) -> &Image {
        self.images.get(handle).expect("Image not found")
    }

    #[inline]
    pub fn get_font(&self, handle: Handle<Font>) -> &Font {
        self.fonts.get(handle).expect("Font not found")
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
