mod atlas;
mod texture;

use crate::atlas::TextureAtlas;
use utils::map::Label;

// Re-exports
pub use texture::Texture;

pub struct AssetManager {
    atlas: TextureAtlas,
}

impl AssetManager {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            atlas: TextureAtlas::new((1024, 1024)),
        }
    }

    #[inline]
    pub fn load_image(&self, label: Label, bytes: Vec<u8>) {
        self.atlas.add_image(label, bytes);
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_texture_coords(&self, label: Label) -> (f32, f32, f32, f32) {
        self.atlas
            .get_region(label)
            .uv_coordinates(self.atlas.size())
    }

    #[inline]
    #[doc(hidden)]
    pub fn get_white_texture_coords(&self) -> (f32, f32, f32, f32) {
        self.atlas.get_white_uv_coords()
    }

    #[inline]
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        self.atlas.texture().bind_group()
    }

    #[inline]
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        self.atlas.bind_group_layout()
    }
}
