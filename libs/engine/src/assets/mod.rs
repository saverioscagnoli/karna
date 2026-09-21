mod atlas;
mod image;
mod packer;
mod worker;

use core::sync::atomic::AtomicBool;

pub use atlas::TextureAtlas;
pub use image::Image;

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
use nostd::sync::channel;
use nostd::thread;
use sdl3::gpu::Device;
use sdl3::image::DecodedImage;
use traccia::error;
use traccia::info;

use crate::assets::worker::worker;

pub use crate::assets::image::ImageRegistry;
pub use crate::assets::worker::AssetThreadPool;

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

#[derive(Debug)]
pub enum AssetSlot<T> {
    Pending,
    Ready(T),
    Failed(String),
}

pub struct AssetServer {
    requests: Sender<AssetRequest>,
    responses: Receiver<AssetResponse>,
    images: ImageRegistry,
}

impl AssetServer {
    pub(crate) fn new(
        requests: Sender<AssetRequest>,
        responses: Receiver<AssetResponse>,
        device: &Device,
    ) -> Self {
        let mut this = Self {
            requests,
            responses,
            images: ImageRegistry::new(device),
        };

        this.images.white_texel = this.bake_image(ImageRegistry::WHITE_TEXEL_BYTES);
        this.images.placeholder = this.bake_image(ImageRegistry::PLACEHOLDER_IMAGE_BYTES);

        this
    }

    pub fn load_image<P>(&mut self, path: P) -> Handle<Image>
    where
        P: AsRef<Path>,
    {
        self.images.load_path(path, &self.requests)
    }

    pub fn load_image_bytes(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.images.load_bytes(bytes.to_vec(), &self.requests)
    }

    pub fn bake_image(&mut self, bytes: &[u8]) -> Handle<Image> {
        self.images.bake(bytes.to_vec())
    }

    pub(crate) fn images(&self) -> &ImageRegistry {
        &self.images
    }

    pub(crate) fn poll(&mut self) {
        while let Ok(r) = self.responses.try_recv() {
            match (r.kind, r.data) {
                (AssetKind::Image, Ok(DecodedAsset::Image(dec))) => {
                    let Some(image) = self.images.atlas.insert(&dec) else {
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
    let assets = AssetServer::new(req_tx, res_rx, device);

    (pool, assets)
}
