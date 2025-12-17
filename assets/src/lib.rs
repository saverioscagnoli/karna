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
}
