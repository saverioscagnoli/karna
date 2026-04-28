use std::sync::Arc;

use math::Size;
use math::Vector4;
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use utils::Handle;

pub use crate::texture_atlas::Image;
use crate::texture_atlas::TextureAtlas;

mod decoding;
mod texture_atlas;

#[derive(Clone)]
pub struct AssetServer {
    atlas: Arc<RwLock<TextureAtlas>>,
}

impl AssetServer {
    pub fn new() -> Self {
        let atlas = TextureAtlas::new(Size::new(1024, 1024));

        Self {
            atlas: Arc::new(RwLock::new(atlas)),
        }
    }

    #[inline]
    pub fn guard(&self) -> AssetServerGuard<'_> {
        AssetServerGuard {
            atlas: self.atlas.read(),
        }
    }

    pub fn load_png(&self, bytes: &[u8]) -> Handle<Image> {
        self.atlas.write().load_image(bytes)
    }
}

pub struct AssetServerGuard<'a> {
    atlas: RwLockReadGuard<'a, TextureAtlas>,
}

impl<'a> AssetServerGuard<'a> {
    #[inline]
    pub fn uv(&self, image: Handle<Image>) -> Vector4 {
        self.atlas.get_uv_coordinates(image)
    }

    #[inline]
    pub fn white_uv(&self) -> Vector4 {
        self.atlas.get_white_uv_coordinates()
    }

    #[inline]
    pub fn image_size(&self, image: Handle<Image>) -> Size<u32> {
        self.atlas.get_image_dimensions(image)
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
}
