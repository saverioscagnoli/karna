mod atlas;
mod image;
mod packer;
mod worker;

use core::cell::Ref;
use core::cell::RefCell;
use core::sync::atomic::AtomicBool;

pub use atlas::TextureAtlas;
pub use image::Image;

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
use sdl3::gpu::Device;
use sdl3::image::DecodedImage;
use traccia::error;
use traccia::info;

use crate::assets::worker::worker;

pub use crate::assets::image::ImageRegistry;
pub use crate::assets::worker::AssetThreadPool;
use crate::text::Font;
use crate::text::TextLayout;
use crate::text::TextSpan;
use crate::text::TextStyle;
use crate::text::TextSystem;

pub enum AssetKind {
    Image,
}

pub enum DecodedAsset {
    Image(DecodedImage),
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
            images: ImageRegistry::new(device),
            text: RefCell::new(TextSystem::default()),
        };

        this.images.white_texel = this.bake_image(ImageRegistry::WHITE_TEXEL_BYTES);
        this.images.placeholder = this.bake_image(ImageRegistry::PLACEHOLDER_IMAGE_BYTES);
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
                }

                (AssetKind::Image, Err(e)) => {
                    error!("Failed to load image: {}", e);
                }
            }
        }

        for request in self.requests.flush() {
            error!("Asset workers are gone, cannot load more assets.");
            self.images.slots[request.slot.cast()] =
                AssetSlot::Failed("Asset worker stopped.".into());
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
