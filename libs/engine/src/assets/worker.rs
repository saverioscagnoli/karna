use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Acquire;
use core::sync::atomic::Ordering::Release;

use nostd::alloc::string::ToString;
use nostd::alloc::sync::Arc;
use nostd::alloc::vec::Vec;
use nostd::fs;
use nostd::path::PathBuf;
use nostd::sync::Receiver;
use nostd::sync::Sender;
use nostd::thread::JoinHandle;
use sdl3::image::DecodedImage;
use traccia::debug;
use traccia::error;

use crate::assets::AssetKind;
use crate::assets::AssetRequest;
use crate::assets::AssetResponse;
use crate::assets::AssetServer;
use crate::assets::AssetSource::Bytes;
use crate::assets::AssetSource::Path;
use crate::assets::DecodedAsset;

pub struct AssetThreadPool {
    pub threads: Vec<JoinHandle<()>>,
    pub cancel: Arc<AtomicBool>,
}

impl AssetThreadPool {
    pub fn shutdown(self, asset_server: AssetServer) {
        self.cancel.store(true, Release);

        drop(asset_server.requests);
        drop(asset_server.responses);
        debug!("Shutting down asset server thread pool.");

        for w in self.threads {
            if w.join().is_none() {
                error!("Asset worker panicked!");
            }
        }
    }
}

pub fn worker(
    root: PathBuf,
    cancel: Arc<AtomicBool>,
    requests: Receiver<AssetRequest>,
    responses: Sender<AssetResponse>,
) {
    while let Ok(req) = requests.recv() {
        if cancel.load(Acquire) {
            debug!("Asset server shutdown, dropping asset request.");
            break;
        }

        let decode = |bytes: &[u8]| match req.kind {
            AssetKind::Image => DecodedImage::from_bytes(bytes).map(DecodedAsset::Image),
        };

        let data = match req.source {
            Path(path) => {
                fs::read(root.join(path)).and_then(|blob| decode(&blob).map_err(|e| e.to_string()))
            }
            Bytes(bytes) => decode(&bytes).map_err(|e| e.to_string()),
        };

        let response = AssetResponse {
            slot: req.slot,
            kind: req.kind,
            data,
        };

        if responses.send(response).is_err() {
            break;
        }
    }
}
