mod atlas;

use std::sync::Arc;

use arc_swap::ArcSwap;
use parking_lot::Mutex;
use sdl3::image::ImageIOStream;
use sdl3::iostream::IOStream;
use sdl3::pixels::PixelFormat;
use utils::Handle;

use crate::assets::atlas::TextureAtlas;

pub use crate::assets::atlas::Image;

#[derive(Debug, Clone)]
pub struct Assets {
    atlas: TextureAtlas,
}

impl Assets {
    fn new() -> Self {
        Self {
            atlas: TextureAtlas::new(),
        }
    }

    pub fn atlas_bind_groups(&self) -> impl Iterator<Item = (u32, &gpu::BindGroup)> + '_ {
        self.atlas.all_bind_groups()
    }

    pub fn atlas_handles(&self) -> impl Iterator<Item = Handle<Image>> + '_ {
        self.atlas.all_handles()
    }

    pub fn load_image(&mut self, data: &[u8]) -> Handle<Image> {
        let io = IOStream::from_bytes(data).expect("Failed to create a stream from image bytes");
        let surface = io.load().expect("Failed to create surface from bytes");
        let surface = surface
            .convert_format(PixelFormat::ABGR8888)
            .expect("Failed to convert image to right format");

        let width = surface.width();
        let height = surface.height();

        let pitch = surface.pitch() as usize;
        let row_bytes = width as usize * 4;

        let data = surface.with_lock(|pixels: &[u8]| {
            let mut out = Vec::with_capacity(row_bytes * height as usize);

            for y in 0..height as usize {
                let start = y * pitch;
                out.extend_from_slice(&pixels[start..start + row_bytes]);
            }
            out
        });

        self.atlas.insert(&data, math::Size::new(width, height))
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

    pub fn get_image(&self, h: Handle<Image>) -> &Image {
        self.snapshot.atlas.get(h)
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
