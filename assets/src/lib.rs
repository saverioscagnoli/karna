use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;

use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;
use utils::Handle;

pub use crate::audio::Audio;
use crate::audio::AudioRegistry;
pub use crate::font::Font;
use crate::font::FontAtlas;
pub use crate::font::GlyphInfo;
pub use crate::font::GlyphKey;
pub use crate::image::Image;
use crate::image::TextureAtlas;

mod audio;
mod font;
mod image;

pub struct Assets {
    textures: TextureAtlas,
    fonts: FontAtlas,
    audios: AudioRegistry,
}

#[derive(Clone)]
pub struct AssetServer {
    assets: Arc<RwLock<Assets>>,
}

impl AssetServer {
    pub fn new() -> Self {
        let mut atlas = TextureAtlas::new((1024, 1024));
        let font_atlas = FontAtlas::new(&mut atlas);

        Self {
            assets: Arc::new(RwLock::new(Assets {
                textures: atlas,
                fonts: font_atlas,
                audios: AudioRegistry::new(),
            })),
        }
    }

    pub fn reader(&self) -> AssetsReader {
        AssetsReader {
            assets: self.assets.clone(),
        }
    }

    pub fn read(&self) -> AssetsRead<'_> {
        utils::profile::count("asset_reads", 1);

        AssetsRead {
            lock: self.assets.read(),
        }
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, Assets> {
        utils::profile::count("asset_writes", 1);

        self.assets.write()
    }

    pub fn write_scope<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut AssetsWrite) -> R,
    {
        utils::profile::count("asset_writes", 1);

        let mut lock = self.assets.write();
        let mut view = AssetsWrite { assets: &mut lock };

        f(&mut view)
    }
}

/// A Clonable handle where only the read methods are implemented.
///
/// For example, a copy of this will be present in each renderer,
/// because when we need to present the frame, we need handles for textures,
/// materials, etc.
pub struct AssetsReader {
    assets: Arc<RwLock<Assets>>,
}

impl AssetsReader {
    pub fn read<'a>(&'a self) -> AssetsRead<'a> {
        utils::profile::count("asset_reads", 1);

        AssetsRead {
            lock: self.assets.read(),
        }
    }
}

/// A read lock over the entire asset server
///
/// Will get this from `AssetsReader::read()`
pub struct AssetsRead<'a> {
    lock: RwLockReadGuard<'a, Assets>,
}

impl<'a> AssetsRead<'a> {
    // ---- Gpu plumbing -----
    // The bind group layout can be cloned for storing, without
    // the need to lock the asset server each time
    //
    // see `Layouts` in `renderer`

    pub fn atlas_bgl(&self) -> &wgpu::BindGroupLayout {
        &self.lock.textures.bgl
    }

    pub fn atlas_bg(&self) -> &wgpu::BindGroup {
        &self.lock.textures.texture.bind_group
    }

    // ---- Metadata ----

    /// Handle for a white pixel in the texture atlas.
    /// When rendering untextured geometry, this need to be bound
    /// in the fragment shader, so that the tex.color is white,
    /// ultimately giving the shape its defined color without the
    /// texture color interfering
    pub fn white_handle(&self) -> Handle<Image> {
        self.lock.textures.white
    }

    /// Handle for the entire texture atlas.
    /// Useful for debugging.
    pub fn atlas_handle(&self) -> Handle<Image> {
        self.lock.textures.handle
    }

    pub fn atlas_size(&self) -> math::Size<u32> {
        self.lock.textures.size
    }

    /// Handle for the debug font.
    /// Internally, `draw.debug_text` uses this.
    pub fn debug_font_handle(&self) -> Handle<Font> {
        self.lock.fonts.debug_font
    }

    // ---- Assets lookups ----

    pub fn get_image(&self, handle: Handle<Image>) -> &Image {
        self.lock
            .textures
            .images
            .get(handle)
            .expect("get_image: stale handle")
    }

    pub fn get_font(&self, handle: Handle<Font>) -> &Font {
        self.lock
            .fonts
            .fonts
            .get(handle)
            .expect("get_font: stale handle")
    }

    pub fn get_glyph(&self, font: Handle<Font>, ch: char, size_px: u16) -> &GlyphInfo {
        let key = GlyphKey { font, ch, size_px };

        self.lock
            .fonts
            .glyphs
            .get(&key)
            .expect("get_glyph: stale handle")
    }

    pub fn get_audio(&self, handle: Handle<Audio>) -> &Audio {
        self.lock
            .audios
            .registry
            .get(handle)
            .expect("get_audio: stale handle")
    }
}

pub struct AssetsWrite<'a> {
    assets: &'a mut Assets,
}

impl<'a> AssetsWrite<'a> {
    pub fn load_image(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.assets.textures.load_image(bytes)
    }

    pub fn load_font(&mut self, bytes: &[u8], size: u16) -> Handle<Font> {
        let font_vec =
            ab_glyph::FontVec::try_from_vec(bytes.to_vec()).expect("Failed to read font");

        self.assets
            .fonts
            .register_font(&mut self.assets.textures, font_vec, size)
    }

    pub fn load_audio(&mut self, bytes: &[u8]) -> Handle<Audio> {
        self.assets.audios.load_audio(bytes)
    }
}
