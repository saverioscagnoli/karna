mod atlas;
mod font;

use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::Mutex;
use sdl3::image::ImageIOStream;
use sdl3::iostream::IOStream;
use sdl3::pixels::PixelFormat;
use sdl3::surface::Surface;
use utils::Handle;
use utils::SlotMap;

use crate::assets::atlas::TextureAtlas;

pub use crate::assets::atlas::Image;
pub use crate::assets::atlas::PagePixels;
pub use crate::assets::font::Font;
pub use crate::assets::font::Glyph;

/// Converts a surface to tightly packed RGBA bytes, dropping row padding.
fn surface_to_rgba(surface: Surface) -> Vec<u8> {
    let surface = surface
        .convert_format(PixelFormat::ABGR8888)
        .expect("Failed to convert surface to right format");

    let height = surface.height() as usize;
    let pitch = surface.pitch() as usize;
    let row_bytes = surface.width() as usize * 4;

    surface.with_lock(|pixels: &[u8]| {
        let mut out = Vec::with_capacity(row_bytes * height);

        for y in 0..height {
            let start = y * pitch;
            out.extend_from_slice(&pixels[start..start + row_bytes]);
        }
        out
    })
}

#[derive(Debug, Clone)]
pub struct Assets {
    atlas: TextureAtlas,
    fonts: SlotMap<Font>,
    debug_font: Handle<Font>,
}

impl Assets {
    fn new() -> Self {
        let mut assets = Self {
            atlas: TextureAtlas::new(),
            fonts: SlotMap::new(),
            debug_font: Handle::INVALID,
        };

        let debug_font = include_bytes!("../../../assets/ProggyClean.ttf");

        assets.debug_font = assets.load_font(debug_font, 24.0);
        assets
    }

    pub fn atlas_pages(&self) -> impl Iterator<Item = &PagePixels> + '_ {
        self.atlas.pages()
    }

    pub fn atlas_page_handles(&self) -> impl Iterator<Item = Handle<Image>> + '_ {
        self.atlas.all_handles()
    }

    pub fn white_pixel(&self) -> &Image {
        self.atlas.white_pixel()
    }

    pub fn debug_font(&self) -> Handle<Font> {
        self.debug_font
    }

    pub fn load_image(&mut self, data: &[u8]) -> Handle<Image> {
        let io = IOStream::from_bytes(data).expect("Failed to create a stream from image bytes");
        let surface = io.load().expect("Failed to create surface from bytes");

        let size = math::Size::new(surface.width(), surface.height());
        let data = surface_to_rgba(surface);

        self.atlas.insert(&data, size)
    }

    pub fn load_font(&mut self, data: &[u8], pt_size: f32) -> Handle<Font> {
        let font = Font::bake(&mut self.atlas, data, pt_size);
        self.fonts.insert(font)
    }

    pub fn get_font(&self, h: Handle<Font>) -> &Font {
        &self.fonts[h]
    }

    pub fn get_image(&self, h: Handle<Image>) -> &Image {
        self.atlas.get(h)
    }
}

struct SharedAssets {
    assets: ArcSwap<Assets>,
    writer_lock: Mutex<()>,
}

pub(crate) struct AssetReader {
    shared: Arc<SharedAssets>,
}

impl AssetReader {
    pub fn snapshot(&self) -> Arc<Assets> {
        self.shared.assets.load_full()
    }
}

#[derive(Clone)]
pub struct AssetServer {
    shared: Arc<SharedAssets>,
}

impl AssetServer {
    pub(crate) fn new() -> Self {
        Self {
            shared: Arc::new(SharedAssets {
                assets: ArcSwap::from_pointee(Assets::new()),
                writer_lock: Mutex::new(()),
            }),
        }
    }

    pub(crate) fn reader(&self) -> AssetReader {
        AssetReader {
            shared: self.shared.clone(),
        }
    }

    fn read_full(&self) -> Arc<Assets> {
        self.shared.assets.load_full()
    }

    fn write_scope<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut Assets) -> R,
    {
        let _w = self.shared.writer_lock.lock();
        let mut view = Assets::clone(&self.shared.assets.load());
        let r = f(&mut view);

        self.shared.assets.store(Arc::new(view));

        r
    }
}

pub struct AssetsView<'a> {
    server: &'a AssetServer,
    snapshot: Arc<Assets>,
}

impl<'a> AssetsView<'a> {
    pub fn new(server: &'a AssetServer) -> Self {
        Self {
            snapshot: server.read_full(),
            server,
        }
    }

    pub fn atlas_page_handles(&self) -> impl Iterator<Item = Handle<Image>> + '_ {
        self.snapshot.atlas_page_handles()
    }

    pub fn white_pixel(&self) -> &Image {
        self.snapshot.white_pixel()
    }

    pub fn debug_font(&self) -> Handle<Font> {
        self.snapshot.debug_font
    }

    pub fn get_image(&self, h: Handle<Image>) -> &Image {
        self.snapshot.atlas.get(h)
    }

    pub fn get_font(&self, h: Handle<Font>) -> &Font {
        self.snapshot.get_font(h)
    }

    pub fn write_scope<R, F>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Assets) -> R,
    {
        let r = self.server.write_scope(f);
        self.snapshot = self.server.read_full();

        r
    }
}
