mod atlas;
mod font;
mod texture;

use crate::atlas::{AtlasRegion, TextureAtlas};
use std::sync::{Arc, RwLock};
use utils::{Label, LabelMap, label};

// Re-exports
pub use font::Font;
pub use texture::Texture;

pub struct AssetManager {
    atlas: TextureAtlas,
    fonts: RwLock<LabelMap<Arc<Font>>>,
}

impl AssetManager {
    #[doc(hidden)]
    pub fn new() -> Self {
        let assets = Self {
            atlas: TextureAtlas::new((1024, 1024)),
            fonts: RwLock::new(LabelMap::default()),
        };

        // Load the default debug font
        assets.load_font(
            label!("debug"),
            include_bytes!("../defaults/DOS-V.ttf").to_vec(),
            16,
        );

        assets
    }

    #[inline]
    pub fn load_image(&self, label: Label, bytes: Vec<u8>) {
        self.atlas.add_image(label, bytes);
    }

    #[inline]
    pub fn load_font(&self, label: Label, bytes: Vec<u8>, size: u8) {
        let mut font = Font::new(bytes, size);
        let mut font_cache = self.fonts.write().expect("Font cache lock is poisoned");

        self.atlas
            .rasterize_characters(label, &mut font, size as f32);
        font_cache.insert(label, Arc::new(font));
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_texture_coords(&self, label: Label) -> (f32, f32, f32, f32) {
        self.atlas
            .get_region(label)
            .uv_coordinates(self.atlas.size())
    }

    #[inline]
    pub fn get_subtexture_coords(
        &self,
        label: Label,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> (f32, f32, f32, f32) {
        let region = self.atlas.get_region(label);
        let atlas_size = self.atlas.size();

        // Calculate normalized coordinates
        let uv_x = (region.x + x) as f32 / atlas_size.width() as f32;
        let uv_y = (region.y + y) as f32 / atlas_size.height() as f32;
        let uv_width = width as f32 / atlas_size.width() as f32;
        let uv_height = height as f32 / atlas_size.height() as f32;

        (uv_x, uv_y, uv_width, uv_height)
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_white_uv_coords(&self) -> (f32, f32, f32, f32) {
        self.atlas.get_white_uv_coords()
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_region(&self, label: Label) -> AtlasRegion {
        self.atlas.get_region(label)
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_font(&self, label: &Label) -> Arc<Font> {
        self.fonts
            .read()
            .expect("Font cache lock is poisoned")
            .get(label)
            .expect("Failed to find font")
            .clone()
    }

    #[inline]
    #[doc(hidden)]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.atlas.texture().bind_group()
    }

    #[inline]
    #[doc(hidden)]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.atlas.bind_group_layout()
    }
}
