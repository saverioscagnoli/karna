use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use crossbeam_channel::unbounded;
use logging::debug;
use logging::error;
use logging::info;

use crate::assets::AssetData;
use crate::assets::AssetKind;
use crate::assets::AssetRequest;
use crate::assets::AssetResponse;
use crate::assets::AssetServer;
use crate::assets::AssetSource;
use crate::assets::image::decode_image;
use crate::assets::image::decode_image_bytes;

pub struct AssetWorkers {
    t: Vec<JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
}

impl AssetWorkers {
    pub fn new(workers: Vec<JoinHandle<()>>, cancel: Arc<AtomicBool>) -> Self {
        Self { t: workers, cancel }
    }

    pub fn shutdown(self, assets: AssetServer) {
        self.cancel.store(true, Ordering::Release);

        #[rustfmt::skip]
        let AssetServer { requests, responses, ..} = assets;

        drop(requests);
        drop(responses);
        debug!("Shutting down asset workers.");

        for w in self.t {
            if w.join().is_err() {
                error!("Asset worker panicked.");
            }
        }
    }
}

fn worker(
    root: PathBuf,
    cancel: Arc<AtomicBool>,
    requests: Receiver<AssetRequest>,
    responses: Sender<AssetResponse>,
) {
    for req in requests {
        if cancel.load(Ordering::Acquire) {
            debug!("Asset worker cancelled, dropping queued request.");
            break;
        }

        let data: Result<AssetData, String>;

        match req.source {
            AssetSource::Path(path) => {
                let path = root.join(&path);

                data = match req.kind {
                    AssetKind::Image(_) => decode_image(&path).map(AssetData::Image),
                    AssetKind::Audio => todo!("Bruh"),
                }
            }
            AssetSource::Bytes(bytes) => {
                data = match req.kind {
                    AssetKind::Image(_) => decode_image_bytes(&bytes).map(AssetData::Image),
                    AssetKind::Audio => todo!("Bruh"),
                }
            }
        }

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

pub fn spawn_workers<P>(root: P, workers: usize) -> (AssetWorkers, AssetServer)
where
    P: AsRef<Path>,
{
    let workers = workers.max(1);
    let root = root.as_ref().to_path_buf();
    debug!("Resolved asset root: {}", root.display());

    let (req_tx, req_rx) = unbounded::<AssetRequest>();
    let (res_tx, res_rx) = unbounded::<AssetResponse>();
    let cancel = Arc::new(AtomicBool::new(false));

    let handles = (0..workers)
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

    info!("Spawned {} worker thread(s) for asset loading.", workers);

    drop(req_rx);
    drop(res_tx);

    let workers = AssetWorkers::new(handles, cancel);
    let assets = AssetServer::new(req_tx, res_rx);

    (workers, assets)
}
