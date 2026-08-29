mod atlas;
mod image;
mod workers;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::mpmc;
use std::sync::mpmc::Receiver;
use std::sync::mpmc::Sender;
use std::thread;

use logging::debug;
use logging::error;
use logging::info;
use utils::ByteSize;
use utils::Handle;

use crate::assets::image::DecodedImage;
use crate::assets::image::ImageRegistry;
use crate::assets::workers::worker;
use crate::gpu::Device;
use crate::gpu::Filter;
use crate::gpu::Texture;
use crate::text::Font;
use crate::text::TextSystem;

pub use crate::assets::atlas::ImageView;
pub use crate::assets::atlas::PageId;
pub use crate::assets::atlas::TextureAtlas;
pub use crate::assets::image::Image;
pub use crate::assets::workers::AssetWorkers;

pub enum AssetKind {
    Image,
}

pub enum DecodedAsset {
    Image(DecodedImage),
}

pub enum AssetSource {
    Path(PathBuf),
    RawBytes(Vec<u8>),
}

pub struct AssetRequest {
    slot: Handle<()>,
    source: AssetSource,
    kind: AssetKind,
}

pub struct AssetResponse {
    slot: Handle<()>,
    kind: AssetKind,
    data: Result<DecodedAsset, String>,
}

#[derive(Debug)]
pub enum AssetSlot<T> {
    Pending,
    Ready(T),
    Failed(String),
}

pub struct AssetServer {
    pub(crate) root: PathBuf,
    pub(crate) requests: Sender<AssetRequest>,
    pub(crate) responses: Receiver<AssetResponse>,
    pub(crate) images: ImageRegistry,
    pub(crate) text: TextSystem,
}

impl AssetServer {
    pub fn new(
        root: PathBuf,
        requests: Sender<AssetRequest>,
        responses: Receiver<AssetResponse>,
    ) -> Self {
        let mut this = Self {
            root,
            requests,
            responses,
            images: ImageRegistry::new(),
            text: TextSystem::default(),
        };

        this.images.white_texel = this.bake_image_bytes(ImageRegistry::WHITE_TEXEL_BYTES);
        this.images.placeholder = this.bake_image_bytes(ImageRegistry::PLACEHOLDER_IMAGE_BYTES);

        this
    }

    pub(crate) fn poll(&mut self, device: &Device) {
        for res in self.responses.try_iter() {
            match (res.kind, res.data) {
                (AssetKind::Image, Ok(DecodedAsset::Image(dec))) => {
                    let filter = Filter::default();
                    let image = self.images.atlas.insert(&dec, res.slot.cast(), filter);

                    info!(
                        "Successfuly loaded image ({:?}, {})",
                        dec.size,
                        ByteSize::from_bytes(dec.rgba.len() as u64)
                    );

                    self.images.slots[res.slot.cast()] = AssetSlot::Ready(image);
                }

                (AssetKind::Image, Err(e)) => {
                    error!("Failed to load image: {}", e);
                    self.images.slots[res.slot.cast()] = AssetSlot::Failed(e.to_string());
                }
            }
        }

        self.images.atlas.upload_dirty(device);
    }

    pub fn white_uv(&self) -> math::Vector2<f32> {
        self.images.get(self.images.white_texel).uv_center()
    }

    pub(crate) fn text_targets(&mut self) -> (&mut TextSystem, &mut TextureAtlas) {
        (&mut self.text, &mut self.images.atlas)
    }

    pub fn load_font<P>(&mut self, path: P) -> Handle<Font>
    where
        P: AsRef<Path>,
    {
        let path = self.root.join(path);

        debug!("Loading font from path '{}'", path.display());

        self.text.register_path(&path)
    }

    pub fn load_font_bytes(&mut self, bytes: &[u8]) -> Handle<Font> {
        debug!(
            "Loading font from raw bytes ({})",
            ByteSize::from_bytes(bytes.len() as u64)
        );

        self.text.register_bytes(bytes)
    }

    pub fn system_font<N>(&mut self, name: N) -> Handle<Font>
    where
        N: AsRef<str>,
    {
        self.text.system(name.as_ref())
    }

    pub fn font_family(&self, font: Handle<Font>) -> Option<&str> {
        self.text.family(font)
    }

    pub(crate) fn white_page(&self) -> PageId {
        self.images.get(self.images.white_texel).page
    }

    pub fn atlas_page_count(&self) -> usize {
        self.images.atlas.page_count()
    }

    pub fn atlas_page_image(&self, index: usize) -> Option<Image> {
        self.images.atlas.page_image(index)
    }

    pub(crate) fn page_texture(&self, page: PageId) -> Option<&Texture> {
        self.images.atlas.page_texture(page)
    }

    pub(crate) fn page_filter(&self, page: PageId) -> Option<Filter> {
        self.images.atlas.page_filter(page)
    }

    /// Upload anything baked since the last poll. Called after the scenes
    /// have run, so text laid out this frame is on the GPU before the frame
    /// that uses it is submitted.
    pub(crate) fn flush(&mut self, device: &Device) {
        self.images.atlas.upload_dirty(device);
    }
}

pub fn spawn(root: PathBuf, workers: usize) -> (AssetWorkers, AssetServer) {
    let workers = workers.max(1);
    let (req_tx, req_rx) = mpmc::channel();
    let (res_tx, res_rx) = mpmc::channel();
    let cancel = Arc::new(AtomicBool::new(false));

    let threads = (0..workers)
        .map(|i| {
            let root = root.clone();
            let cancel = cancel.clone();
            let requests = req_rx.clone();
            let responses = res_tx.clone();

            thread::Builder::new()
                .name(format!("asset-worker-{i}"))
                .spawn(move || worker(root, cancel, requests, responses))
                .expect("Failed to spawn asset worker")
        })
        .collect::<Vec<_>>();

    info!("Spawned {} worker thread(s) for asset decoding.", workers);

    drop(req_rx);
    drop(res_tx);

    let workers = AssetWorkers { threads, cancel };
    let asset_server = AssetServer::new(root, req_tx, res_rx);

    (workers, asset_server)
}
