mod atlas;
mod audio;
mod geometry;
mod image;
mod material;
mod packer;
mod worker;

use core::cell::Ref;
use core::cell::RefCell;
use core::sync::atomic::AtomicBool;

use nostd::alloc::collections::VecDeque;
use nostd::alloc::ffi::CString;
use nostd::alloc::format;
use nostd::alloc::string::String;
use nostd::alloc::sync::Arc;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::path::Path;
use nostd::path::PathBuf;
use nostd::sync::Receiver;
use nostd::sync::Sender;
use nostd::sync::TrySendError;
use nostd::sync::channel;
use nostd::thread;
use sdl3::audio::AudioSpec;
use sdl3::gpu::Device;
use sdl3::image::DecodedImage;
use traccia::error;
use traccia::info;

use crate::assets::geometry::GeometryRegistry;
use crate::assets::material::MaterialRegistry;
use crate::assets::worker::worker;

use crate::mesh::Geometry;
use crate::mesh::Material;
use crate::text::Font;
use crate::text::TextLayout;
use crate::text::TextSpan;
use crate::text::TextStyle;
use crate::text::TextSystem;

pub use crate::assets::atlas::TextureAtlas;
pub use crate::assets::audio::Audio;
pub use crate::assets::audio::AudioData;
pub use crate::assets::audio::AudioKind;
pub use crate::assets::audio::AudioRegistry;
pub use crate::assets::image::Image;
pub use crate::assets::image::ImageRegistry;
pub use crate::assets::worker::AssetThreadPool;

pub enum AssetKind {
    Image,
    Audio(AudioKind),
}

pub enum DecodedAsset {
    Image(DecodedImage),
    Audio(AudioSpec, AudioData),
}

pub enum AssetSource {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

pub struct AssetRequest {
    slot: Handle<()>,
    kind: AssetKind,
    source: AssetSource,
}

pub struct AssetResponse {
    slot: Handle<()>,
    kind: AssetKind,
    data: Result<DecodedAsset, String>,
}

pub struct AssetQueue {
    sender: Sender<AssetRequest>,
    backlog: VecDeque<AssetRequest>,
}

impl AssetQueue {
    fn new(sender: Sender<AssetRequest>) -> Self {
        Self {
            sender,
            backlog: VecDeque::new(),
        }
    }

    pub(crate) fn submit(&mut self, request: AssetRequest) -> Result<(), AssetRequest> {
        if !self.backlog.is_empty() {
            self.backlog.push_back(request);
            return Ok(());
        }

        match self.sender.try_send(request) {
            Ok(()) => Ok(()),
            Err(TrySendError::Full(request)) => {
                self.backlog.push_back(request);
                Ok(())
            }
            Err(TrySendError::Closed(request)) => Err(request),
        }
    }

    fn flush(&mut self) -> Vec<AssetRequest> {
        while let Some(request) = self.backlog.pop_front() {
            match self.sender.try_send(request) {
                Ok(()) => {}
                Err(TrySendError::Full(request)) => {
                    self.backlog.push_front(request);
                    break;
                }
                Err(TrySendError::Closed(request)) => {
                    self.backlog.push_front(request);
                    return self.backlog.drain(..).collect();
                }
            }
        }

        Vec::new()
    }
}

#[derive(Debug)]
pub enum AssetSlot<T> {
    Pending,
    Ready(T),
    Failed(String),
}

pub struct AssetServer {
    root: PathBuf,
    requests: AssetQueue,
    responses: Receiver<AssetResponse>,
    images: ImageRegistry,
    audios: AudioRegistry,
    geometries: GeometryRegistry,
    materials: MaterialRegistry,
    text: RefCell<TextSystem>,
}

impl AssetServer {
    pub(crate) fn new(
        root: PathBuf,
        requests: Sender<AssetRequest>,
        responses: Receiver<AssetResponse>,
        device: &Device,
    ) -> Self {
        let mut this = Self {
            root,
            requests: AssetQueue::new(requests),
            responses,
            images: ImageRegistry::new(device.share()),
            audios: AudioRegistry::default(),
            geometries: GeometryRegistry::new(device.share()),
            materials: MaterialRegistry::new(),
            text: RefCell::new(TextSystem::default()),
        };

        this.images.white_texel = this.bake_image(ImageRegistry::WHITE_TEXEL_BYTES);
        this.images.placeholder = this.bake_image(ImageRegistry::PLACEHOLDER_IMAGE_BYTES);
        this.audios.fallback = this.bake_audio(AudioRegistry::SILENCE_BYTES);

        let debug_font =
            this.load_font_bytes_sized(TextSystem::DEBUG_FONT_BYTES, TextSystem::DEBUG_FONT_SIZE);
        let text = this.text.get_mut();

        text.debug_font = debug_font;
        text.set_default_font(debug_font);

        this
    }

    /// The directory asset paths are resolved against.
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load_image<P>(&mut self, path: P) -> Handle<Image>
    where
        P: AsRef<Path>,
    {
        self.images.load_path(path, &mut self.requests)
    }

    pub fn load_image_bytes(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.images.load_bytes(bytes.to_vec(), &mut self.requests)
    }

    pub fn bake_image(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.images.bake(bytes.to_vec())
    }

    pub fn load_audio<P>(&mut self, path: P) -> Handle<Audio>
    where
        P: AsRef<Path>,
    {
        self.audios
            .load_path(path, AudioKind::Oneshot, &mut self.requests)
    }

    pub fn load_audio_bytes(&mut self, bytes: &[u8]) -> Handle<Audio> {
        self.audios
            .load_bytes(bytes.to_vec(), AudioKind::Oneshot, &mut self.requests)
    }

    pub fn load_audio_stream<P>(&mut self, path: P) -> Handle<Audio>
    where
        P: AsRef<Path>,
    {
        self.audios
            .load_path(path, AudioKind::Streaming, &mut self.requests)
    }

    pub fn load_audio_stream_bytes(&mut self, bytes: &[u8]) -> Handle<Audio> {
        self.audios
            .load_bytes(bytes.to_vec(), AudioKind::Streaming, &mut self.requests)
    }

    pub fn bake_audio(&mut self, bytes: &[u8]) -> Handle<Audio> {
        self.audios.bake(bytes.to_vec(), AudioKind::Oneshot)
    }

    pub fn bake_audio_stream(&mut self, bytes: &[u8]) -> Handle<Audio> {
        self.audios.bake(bytes.to_vec(), AudioKind::Streaming)
    }

    pub fn audio(&self, handle: Handle<Audio>) -> &Audio {
        self.audios.get(handle)
    }

    pub fn try_audio(&self, handle: Handle<Audio>) -> Option<&Audio> {
        self.audios.resolve(handle)
    }

    pub fn add_geometry(&mut self, geometry: Geometry) -> Handle<Geometry> {
        self.geometries.add(geometry)
    }

    #[track_caller]
    pub fn geometry(&self, handle: Handle<Geometry>) -> &Geometry {
        self.geometries
            .get(handle)
            .unwrap_or_else(|| panic!("Geometry {:?} not found.", handle))
    }

    #[track_caller]
    pub fn geometry_mut(&mut self, handle: Handle<Geometry>) -> &mut Geometry {
        self.geometries
            .get_mut(handle)
            .unwrap_or_else(|| panic!("Geometry {:?} not found.", handle))
    }

    pub fn remove_geometry(&mut self, handle: Handle<Geometry>) -> Option<Geometry> {
        self.geometries.remove(handle)
    }

    pub fn add_material(&mut self, material: Material) -> Handle<Material> {
        self.materials.add(material)
    }

    #[track_caller]
    pub fn material(&self, handle: Handle<Material>) -> &Material {
        self.materials
            .get(handle)
            .unwrap_or_else(|| panic!("Material {:?} not found.", handle))
    }

    #[track_caller]
    pub fn material_mut(&mut self, handle: Handle<Material>) -> &mut Material {
        self.materials
            .get_mut(handle)
            .unwrap_or_else(|| panic!("Material {:?} not found.", handle))
    }

    pub fn remove_material(&mut self, handle: Handle<Material>) -> Option<Material> {
        self.materials.remove(handle)
    }

    pub fn load_font<P>(&mut self, path: P) -> Handle<Font>
    where
        P: AsRef<Path>,
    {
        self.load_font_sized(path, TextSystem::DEFAULT_FONT_SIZE)
    }

    pub fn load_font_sized<P>(&mut self, path: P, size: f32) -> Handle<Font>
    where
        P: AsRef<Path>,
    {
        let path = self.root.join(path.as_ref());
        self.text.get_mut().register_path(&path, size)
    }

    pub fn load_font_bytes(&mut self, bytes: &[u8]) -> Handle<Font> {
        self.load_font_bytes_sized(bytes, TextSystem::DEFAULT_FONT_SIZE)
    }

    pub fn load_font_bytes_sized(&mut self, bytes: &[u8], size: f32) -> Handle<Font> {
        self.text.get_mut().register_bytes(bytes, size)
    }

    pub fn debug_font(&self) -> Handle<Font> {
        self.text.borrow().debug_font
    }

    pub fn placeholder_image(&self) -> Handle<Image> {
        self.images.placeholder
    }

    pub(crate) fn layout(&self, spans: &[TextSpan], style: &TextStyle) -> Arc<TextLayout> {
        self.text
            .borrow_mut()
            .layout(spans, style, &mut self.images.atlas.borrow_mut())
    }

    pub(crate) fn layout_str(&self, text: &str, style: &TextStyle) -> Arc<TextLayout> {
        self.text
            .borrow_mut()
            .layout_str(text, style, &mut self.images.atlas.borrow_mut())
    }

    pub(crate) fn atlas(&self) -> Ref<'_, TextureAtlas> {
        self.images.atlas.borrow()
    }

    pub(crate) fn images(&self) -> &ImageRegistry {
        &self.images
    }

    pub(crate) fn geometries(&self) -> &GeometryRegistry {
        &self.geometries
    }

    pub(crate) fn materials(&self) -> &MaterialRegistry {
        &self.materials
    }

    pub(crate) fn upload_geometries(&mut self) {
        self.geometries.upload_dirty();
    }

    pub(crate) fn text_mut(&mut self) -> &mut TextSystem {
        self.text.get_mut()
    }

    pub(crate) fn poll(&mut self) {
        while let Ok(r) = self.responses.try_recv() {
            match (r.kind, r.data) {
                (AssetKind::Image, Ok(DecodedAsset::Image(dec))) => {
                    let Some(image) = self.images.atlas.get_mut().insert(&dec) else {
                        error!("Failed to pack image into texture atlas");
                        continue;
                    };

                    info!("Loaded image {:?} (page {})", image.size(), image.page());
                    self.images.slots[r.slot.cast()] = AssetSlot::Ready(image);
                    self.images.generation += 1;
                }

                (AssetKind::Image, Err(e)) => {
                    error!("Failed to load image: {}", e);
                    self.images.slots[r.slot.cast()] = AssetSlot::Failed(e);
                }

                (AssetKind::Audio(kind), Ok(DecodedAsset::Audio(spec, data))) => {
                    info!("Loaded sound {:?}, spec {:?}", kind, spec);

                    let sound = Audio::new(spec, data);
                    self.audios.slots[r.slot.cast()] = AssetSlot::Ready(sound)
                }

                (AssetKind::Audio(_), Err(e)) => {
                    error!("Failed to load audio: {}", e);
                    self.audios.slots[r.slot.cast()] = AssetSlot::Failed(e);
                }

                _ => unreachable!(),
            }
        }

        for request in self.requests.flush() {
            error!("Asset workers are gone, cannot load more assets.");

            match request.kind {
                AssetKind::Image => {
                    self.images.slots[request.slot.cast()] =
                        AssetSlot::Failed("Asset worker stopped.".into());
                }
                AssetKind::Audio(_) => {
                    self.audios.slots[request.slot.cast()] =
                        AssetSlot::Failed("Asset worker stopped.".into());
                }
            }
        }
    }
}

pub fn spawn(root: PathBuf, workers: usize, device: &Device) -> (AssetThreadPool, AssetServer) {
    let workers = workers.max(1);
    let (req_tx, req_rx) = channel(8).expect("Failed to create channel");
    let (res_tx, res_rx) = channel(8).expect("Failed to create channel");
    let cancel = Arc::new(AtomicBool::new(false));

    let threads = (0..workers)
        .map(|i| {
            let root = root.clone();
            let cancel = Arc::clone(&cancel);
            let requests = req_rx.clone();
            let responses = res_tx.clone();

            thread::spawn(
                &CString::new(format!("asset-worker-{i}")).unwrap_or(c"asset-worker".into()),
                move || worker(root, cancel, requests, responses),
            )
            .expect("Failed to spawn thread")
        })
        .collect::<Vec<_>>();

    info!("Spawned {} worker thread(s) for asset loading.", workers);

    drop(req_rx);
    drop(res_tx);

    let pool = AssetThreadPool { threads, cancel };
    let assets = AssetServer::new(root, req_tx, res_rx, device);

    (pool, assets)
}
