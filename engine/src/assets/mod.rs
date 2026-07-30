pub mod atlas;
pub mod audio;
pub mod font;
pub mod image;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::thread::{self};

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use crossbeam_channel::unbounded;
use gpu::Filter;
use gpu::Gpu;
use logging::debug;
use logging::error;
use logging::fatal;
use logging::info;
use logging::warn;
use sdl3::pixels::PixelFormat;
use sdl3::surface::Surface;
use utils::ByteSize;
use utils::FastHashMap;
use utils::Handle;
use utils::SlotMap;

use crate::assets::atlas::TextureAtlas;
use crate::assets::audio::Audio;
use crate::assets::audio::decode_audio;
use crate::assets::font::DecodedFont;
use crate::assets::font::Font;
use crate::assets::font::Rasterize;
use crate::assets::font::decode_font;
use crate::assets::font::decode_font_bytes;
use crate::assets::image::DecodedImage;
use crate::assets::image::Image;
use crate::assets::image::decode_image;
use crate::assets::image::decode_image_bytes;
use crate::sound::SharedMixer;

const WHITE_PIXEL_BYTES: [u8; 4] = [255, 255, 255, 255];
const PLACEHOLDER_IMAGE_BYTES: &'static [u8] = include_bytes!("../../../assets/placeholder.png");
const DEBUG_FONT_BYTES: &'static [u8] = include_bytes!("../../../assets/ProggyClean.ttf");
const DEBUG_FONT_SIZE: f32 = 32.0;

pub enum AssetKind {
    Image(Filter),
    Audio,
    Font(f32, Rasterize),
}

pub enum AssetData {
    Image(DecodedImage),
    Font(DecodedFont),
    Audio(Audio),
}

pub struct AssetRequest {
    slot: Handle<()>,
    path: PathBuf,
    kind: AssetKind,
}

pub struct AssetResponse {
    slot: Handle<()>,
    kind: AssetKind,
    data: Result<AssetData, String>,
}

#[derive(Debug)]
pub enum AssetSlot<T> {
    Pending,
    Ready(T),
    Failed,
}

pub struct Assets {
    images: SlotMap<AssetSlot<Image>>,
    image_paths: FastHashMap<(PathBuf, Filter), Handle<Image>>,
    atlas: TextureAtlas,
    white: Handle<Image>,
    placeholder: Handle<Image>,
    fonts: SlotMap<AssetSlot<Font>>,
    font_paths: FastHashMap<(PathBuf, u32, Rasterize), Handle<Font>>,
    debug_font: Handle<Font>,
    audios: SlotMap<AssetSlot<Audio>>,
    audio_paths: FastHashMap<PathBuf, Handle<Audio>>,
    requests: Sender<AssetRequest>,
    responses: Receiver<AssetResponse>,
}

impl Assets {
    fn new(requests: Sender<AssetRequest>, responses: Receiver<AssetResponse>) -> Self {
        let mut this = Self {
            images: SlotMap::new(),
            image_paths: FastHashMap::default(),
            atlas: TextureAtlas::new(),
            white: Handle::INVALID,
            placeholder: Handle::INVALID,
            fonts: SlotMap::new(),
            font_paths: FastHashMap::default(),
            debug_font: Handle::INVALID,
            audios: SlotMap::new(),
            audio_paths: FastHashMap::default(),
            requests,
            responses,
        };

        let white = this.load_image_raw(&WHITE_PIXEL_BYTES, math::Size::new(1, 1));
        let placeholder = this.load_image_bytes(PLACEHOLDER_IMAGE_BYTES);
        let debug_font =
            this.load_font_bytes_with(DEBUG_FONT_BYTES, DEBUG_FONT_SIZE, Rasterize::Crisp);

        this.white = white;
        this.placeholder = placeholder;
        this.debug_font = debug_font;

        this
    }

    pub fn load_image<P>(&mut self, path: P) -> Handle<Image>
    where
        P: AsRef<Path>,
    {
        self.load_image_with(path, Filter::default())
    }

    /// Load an image sampled with `filter` instead of the default.
    pub fn load_image_with<P>(&mut self, path: P, filter: Filter) -> Handle<Image>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let key = (path.to_path_buf(), filter);

        if let Some(handle) = self.image_paths.get(&key) {
            return *handle;
        }

        let handle: Handle<Image> = self.images.insert(AssetSlot::Pending).cast();
        self.image_paths.insert(key, handle);

        let request = AssetRequest {
            slot: handle.cast(),
            path: path.to_path_buf(),
            kind: AssetKind::Image(filter),
        };

        if self.requests.send(request).is_err() {
            warn!("Asset workers are gone, cannot load '{}'", path.display());
            self.images[handle.cast()] = AssetSlot::Failed;
        }

        handle
    }

    // TODO: Maybe make something like an AssetSource(File, Bytes) and send that to worker
    pub fn load_image_bytes(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.load_image_bytes_with(bytes, Filter::default())
    }

    pub fn load_image_bytes_with(&mut self, bytes: &[u8], filter: Filter) -> Handle<Image> {
        let handle: Handle<Image> = self.images.insert(AssetSlot::Pending).cast();

        match decode_image_bytes(bytes) {
            Ok(dec) => {
                let image = self.atlas.insert(&dec, handle, filter);
                self.images[handle.cast()] = AssetSlot::Ready(image);
            }
            Err(e) => {
                error!("Failed to decode image bytes: {e}");
                self.images[handle.cast()] = AssetSlot::Failed;
            }
        }

        handle
    }

    pub fn load_image_raw(&mut self, pixels: &[u8], size: math::Size<u32>) -> Handle<Image> {
        self.load_image_raw_with(pixels, size, Filter::default())
    }

    pub fn load_image_raw_with(
        &mut self,
        pixels: &[u8],
        size: math::Size<u32>,
        filter: Filter,
    ) -> Handle<Image> {
        let handle: Handle<Image> = self.images.insert(AssetSlot::Pending).cast();
        let image = self.atlas.insert_rgba(pixels, size, handle, filter);

        self.images[handle.cast()] = AssetSlot::Ready(image);

        handle
    }

    pub fn get_image(&self, handle: Handle<Image>) -> &Image {
        if let Some(AssetSlot::Ready(image)) = self.images.get(handle.cast()) {
            return image;
        }

        match self.images.get(self.placeholder.cast()) {
            Some(AssetSlot::Ready(image)) => image,
            _ => fatal!("Placeholder image is missing from the atlas"),
        }
    }

    pub fn atlas_page_count(&self) -> usize {
        self.atlas.page_count()
    }

    pub fn white_pixel(&self) -> &Image {
        self.get_image(self.white)
    }

    pub fn placeholder(&self) -> &Image {
        self.get_image(self.placeholder)
    }

    pub(crate) fn page_texture(&self, index: usize) -> Option<&gpu::Texture> {
        self.atlas.page_texture(index)
    }

    pub fn load_font<P>(&mut self, path: P, point_size: f32) -> Handle<Font>
    where
        P: AsRef<Path>,
    {
        self.load_font_with(path, point_size, Rasterize::default())
    }

    /// Load a font rasterized with `mode` instead of the default.
    pub fn load_font_with<P>(&mut self, path: P, point_size: f32, mode: Rasterize) -> Handle<Font>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let key = (path.to_path_buf(), point_size.to_bits(), mode);

        if let Some(handle) = self.font_paths.get(&key) {
            return *handle;
        }

        let handle: Handle<Font> = self.fonts.insert(AssetSlot::Pending).cast();
        self.font_paths.insert(key, handle);

        let request = AssetRequest {
            slot: handle.cast(),
            path: path.to_path_buf(),
            kind: AssetKind::Font(point_size, mode),
        };

        if self.requests.send(request).is_err() {
            warn!("Asset workers are gone, cannot load '{}'", path.display());
            self.images[handle.cast()] = AssetSlot::Failed;
        }

        handle
    }

    // TODO: Maybe make something like an AssetSource(File, Bytes) and send that to worker
    pub fn load_font_bytes(&mut self, bytes: &[u8], point_size: f32) -> Handle<Font> {
        self.load_font_bytes_with(bytes, point_size, Rasterize::default())
    }

    pub fn load_font_bytes_with(
        &mut self,
        bytes: &[u8],
        point_size: f32,
        mode: Rasterize,
    ) -> Handle<Font> {
        let handle: Handle<Font> = self.fonts.insert(AssetSlot::Pending).cast();

        match decode_font_bytes(bytes, point_size, mode) {
            Ok(dec) => {
                let font = Font::bake(&mut self.atlas, &mut self.images, dec);
                self.fonts[handle.cast()] = AssetSlot::Ready(font);
            }
            Err(e) => {
                error!("Failed to decode font bytes: {e}");
                self.fonts[handle.cast()] = AssetSlot::Failed;
            }
        }

        handle
    }

    pub fn get_font(&self, handle: Handle<Font>) -> &Font {
        match self.fonts.get(handle.cast()) {
            Some(AssetSlot::Ready(f)) => f,
            _ => match self.fonts.get(self.debug_font.cast()) {
                Some(AssetSlot::Ready(f)) => f,
                _ => fatal!("No debug font is set"),
            },
        }
    }

    pub fn debug_font(&self) -> Handle<Font> {
        self.debug_font
    }

    pub fn load_audio<P: AsRef<Path>>(&mut self, path: P) -> Handle<Audio> {
        let path = path.as_ref();

        if let Some(handle) = self.audio_paths.get(path) {
            return *handle;
        }

        let handle: Handle<Audio> = self.audios.insert(AssetSlot::Pending).cast();
        self.audio_paths.insert(path.to_path_buf(), handle);

        let request = AssetRequest {
            slot: handle.cast(),
            path: path.to_path_buf(),
            kind: AssetKind::Audio,
        };

        if self.requests.send(request).is_err() {
            warn!("Asset workers are gone, cannot load '{}'", path.display());
            self.audios[handle.cast()] = AssetSlot::Failed;
        }

        handle
    }

    pub fn get_audio(&self, handle: Handle<Audio>) -> &Audio {
        match self.audios.get(handle.cast()) {
            Some(AssetSlot::Ready(a)) => a,
            _ => fatal!("Cannot get audio"),
        }
    }

    pub(crate) fn poll(&mut self, gpu: &Gpu) {
        for res in self.responses.try_iter() {
            let slot: Handle<Image> = res.slot.cast();

            match (res.kind, res.data) {
                (AssetKind::Image(filter), Ok(AssetData::Image(dec))) => {
                    let image = self.atlas.insert(&dec, slot, filter);

                    debug!(
                        "Loaded image ({:?}, {})",
                        dec.size,
                        ByteSize::from_bytes(dec.pixels.len() as u64)
                    );

                    if let Some(entry) = self.images.get_mut(slot.cast()) {
                        *entry = AssetSlot::Ready(image);
                    }
                }

                (AssetKind::Image(_), Err(e)) => {
                    error!("Failed to decode image: {e}");

                    if let Some(entry) = self.images.get_mut(slot.cast()) {
                        *entry = AssetSlot::Failed;
                    }
                }

                (AssetKind::Audio, Ok(AssetData::Audio(dec))) => {
                    debug!("Loaded audio (duration: {:?})", dec.length());

                    if let Some(entry) = self.audios.get_mut(slot.cast()) {
                        *entry = AssetSlot::Ready(dec);
                    }
                }

                (AssetKind::Audio, Err(e)) => {
                    error!("Failed to decode audio: {}", e);

                    if let Some(entry) = self.audios.get_mut(slot.cast()) {
                        *entry = AssetSlot::Failed;
                    }
                }

                (AssetKind::Font(..), Ok(AssetData::Font(dec))) => {
                    debug!(
                        "Loaded font ({}, rasterized in {:?})",
                        dec.name, dec.duration
                    );

                    let font = Font::bake(&mut self.atlas, &mut self.images, dec);

                    if let Some(entry) = self.fonts.get_mut(slot.cast()) {
                        *entry = AssetSlot::Ready(font);
                    }
                }

                (AssetKind::Font(..), Err(e)) => {
                    error!("Failed to decode font: {}", e);

                    if let Some(entry) = self.fonts.get_mut(slot.cast()) {
                        *entry = AssetSlot::Failed;
                    }
                }

                _ => unreachable!(),
            }
        }

        self.atlas.upload_dirty(gpu);
    }
}

pub struct AssetServer {
    workers: Vec<JoinHandle<()>>,
}

impl AssetServer {
    pub fn new(workers: Vec<JoinHandle<()>>) -> Self {
        Self { workers }
    }
    pub fn shutdown(self, assets: Assets) {
        drop(assets);
        debug!("Shutting down asset server.");

        for w in self.workers {
            let _ = w.join();
        }
    }
}

fn worker(
    root: PathBuf,
    requests: Receiver<AssetRequest>,
    responses: Sender<AssetResponse>,
    mixer: Arc<SharedMixer>,
) {
    for req in requests {
        let path = root.join(&req.path);
        let data = match req.kind {
            AssetKind::Image(_) => decode_image(&path).map(AssetData::Image),
            AssetKind::Font(pt, mode) => decode_font(&path, pt, mode).map(AssetData::Font),
            AssetKind::Audio => decode_audio(&mixer, &path).map(AssetData::Audio),
        };

        if responses
            .send(AssetResponse {
                slot: req.slot,
                kind: req.kind,
                data,
            })
            .is_err()
        {
            break;
        }
    }
}

pub fn spawn<P>(root: P, workers: usize, mixer: Arc<SharedMixer>) -> (AssetServer, Assets)
where
    P: AsRef<Path>,
{
    let root = root.as_ref().to_path_buf();

    debug!("Resolved assets root: {}", root.display());

    let (req_tx, req_rx) = unbounded::<AssetRequest>();
    let (res_tx, res_rx) = unbounded::<AssetResponse>();

    let handles = (0..workers.max(1))
        .map(|i| {
            let mixer = Arc::clone(&mixer);
            let root = root.clone();
            let requests = req_rx.clone();
            let responses = res_tx.clone();

            thread::Builder::new()
                .name(format!("asset-worker-{i}"))
                .spawn(move || worker(root, requests, responses, mixer))
                .expect("Failed to spawn asset worker")
        })
        .collect::<Vec<_>>();

    info!("Spawned {} worker thread(s) for decoding assets.", workers);

    let server = AssetServer::new(handles);
    let assets = Assets::new(req_tx, res_rx);

    drop(req_rx);
    drop(res_tx);

    (server, assets)
}

pub fn resolve_base_path() -> PathBuf {
    match sdl3::filesystem::get_base_path() {
        Ok(p) => return PathBuf::from(p),
        Err(e) => warn!("SDL_GetBasePath failed ({e}), falling back to current_exe"),
    }

    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => fatal!("Failed to resolve base path: {}", e),
    };

    match exe.parent() {
        Some(dir) => dir.to_path_buf(),
        None => fatal!("Executable path has no parent directory: {}", exe.display()),
    }
}

pub fn surface_to_rgba(surface: Surface) -> Result<Vec<u8>, String> {
    let surface = surface
        .convert_format(PixelFormat::ABGR8888)
        .map_err(|e| e.to_string())?;

    let height = surface.height() as usize;
    let pitch = surface.pitch() as usize;
    let row_bytes = surface.width() as usize * 4;

    let data = surface.with_lock(|pixels: &[u8]| {
        let mut out = Vec::with_capacity(row_bytes * height);

        for y in 0..height {
            let start = y * pitch;
            out.extend_from_slice(&pixels[start..start + row_bytes]);
        }
        out
    });

    Ok(data)
}
