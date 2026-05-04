mod decoding;
mod font;
mod texture_atlas;

use std::sync::Arc;

use logging::info;
use math::Size;
use math::Vector4;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use utils::ByteSize;
use utils::Handle;
use utils::SlotMap;

pub use crate::font::Font;
pub use crate::texture_atlas::Image;
use crate::texture_atlas::TextureAtlas;

#[derive(Clone)]
pub struct AssetServer {
    atlas: Arc<RwLock<TextureAtlas>>,
    fonts: Arc<RwLock<SlotMap<Font>>>,
    debug_font: Handle<Font>,
}

impl AssetServer {
    pub fn new() -> Self {
        let atlas = TextureAtlas::new(Size::new(1024, 1024));

        let mut assset_server = Self {
            atlas: Arc::new(RwLock::new(atlas)),
            fonts: Arc::new(RwLock::new(SlotMap::new())),
            debug_font: Handle::default(),
        };

        assset_server.debug_font =
            assset_server.load_font(include_bytes!("../../fonts/DOS-V.ttf"), 16);

        assset_server
    }

    #[inline]
    pub fn guard(&self) -> AssetServerGuard<'_> {
        AssetServerGuard {
            atlas: self.atlas.read(),
            fonts: self.fonts.read(),
        }
    }

    pub fn load_png(&self, bytes: &[u8]) -> Handle<Image> {
        self.atlas.write().load_image(bytes)
    }

    pub fn load_font(&self, bytes: &[u8], size: u16) -> Handle<Font> {
        let mut atlas = self.atlas.write();
        let mut font_map = self.fonts.write();

        let mut font = Font::new(bytes, size);

        atlas.register_font(&mut font);

        info!(
            "Loaded font with size={}",
            ByteSize::from_bytes(bytes.len() as u64)
        );

        font_map.insert(font)
    }
}

pub struct AssetServerGuard<'a> {
    atlas: RwLockReadGuard<'a, TextureAtlas>,
    fonts: RwLockReadGuard<'a, SlotMap<Font>>,
}

impl<'a> AssetServerGuard<'a> {
    #[inline]
    pub fn white_pixel(&self) -> &Image {
        self.get_image(self.atlas.white_pixel_handle)
    }

    #[inline]
    pub fn atlas_handle(&self) -> Handle<Image> {
        self.atlas.handle
    }

    #[inline]
    pub fn atlas_size(&self) -> &Size<u32> {
        self.atlas.size()
    }

    #[inline]
    pub fn get_image(&self, handle: Handle<Image>) -> &Image {
        self.atlas.get_image(handle)
    }

    /// Bind group layout for the texture atlas.
    ///
    /// This matches the atlas shader bindings:
    /// - @binding(0): texture_2d<f32>
    /// - @binding(1): sampler
    #[inline]
    pub fn atlas_bgl(&self) -> &wgpu::BindGroupLayout {
        self.atlas.bind_group_layout()
    }

    /// Bind group containing the atlas texture view + sampler.
    #[inline]
    pub fn atlas_bg(&self) -> &wgpu::BindGroup {
        self.atlas.bind_group()
    }

    #[inline]
    pub fn get_font(&self, handle: Handle<Font>) -> &Font {
        self.fonts.get(handle).expect("Failed to get font")
    }
}
