use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpmc::Receiver;
use std::sync::mpmc::Sender;
use std::thread::JoinHandle;

use logging::debug;
use logging::error;

use crate::Key::P;
use crate::assets::AssetKind;
use crate::assets::AssetRequest;
use crate::assets::AssetResponse;
use crate::assets::AssetServer;
use crate::assets::AssetSource;
use crate::assets::DecodedAsset;
use crate::assets::image::decode_image;
use crate::assets::image::decode_image_bytes;

pub struct AssetWorkers {
    pub threads: Vec<JoinHandle<()>>,
    pub cancel: Arc<AtomicBool>,
}

impl AssetWorkers {
    pub fn shutdown(self, asset_server: AssetServer) {
        self.cancel.store(true, Ordering::Release);

        #[rustfmt::skip]
        let AssetServer { requests, responses, .. }  = asset_server;

        drop(requests);
        drop(responses);
        debug!("Shutting down asset workers.");

        for w in self.threads {
            if w.join().is_err() {
                error!("Asset worker panicked.");
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
    for req in requests {
        if cancel.load(Ordering::Acquire) {
            debug!("Asset worker stopped, dropping queued asset requests.");
            break;
        }

        let data = match req.source {
            AssetSource::Path(path) => {
                let path = root.join(path);

                match req.kind {
                    AssetKind::Image => decode_image(&path).map(DecodedAsset::Image),
                }
            }

            AssetSource::RawBytes(bytes) => match req.kind {
                AssetKind::Image => decode_image_bytes(&bytes).map(DecodedAsset::Image),
            },
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
