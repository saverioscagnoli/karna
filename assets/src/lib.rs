use std::sync::Arc;

use math::Size;
use parking_lot::RwLock;
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

    pub fn load_png(&self, bytes: &[u8]) -> Handle<Image> {
        self.atlas.write().load_image(bytes)
    }
}
