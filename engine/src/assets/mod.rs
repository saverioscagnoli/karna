use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::AtomicU32,
        mpsc::{Receiver, Sender},
    },
};

use parking_lot::RwLock;
use utils::FastHashMap;

#[derive(Debug, Clone, Copy)]
pub enum AssetKind {
    Texture,
    Audio,
}

#[derive(Debug, Clone)]
pub struct AssetRequest {
    pub id: u32,
    pub path: Arc<str>,
    pub kind: AssetKind,
}

#[derive(Debug, Clone)]
pub enum AssetPayload {
    Image {
        width: u32,
        height: u32,
        data: Vec<u8>,
    },
    Audio {
        channels: u16,
        samples: Vec<f32>,
    },
}

pub struct AssetUpload {
    pub id: u32,
    pub payload: AssetPayload,
}

pub enum AssetMeta {
    Image { width: u32, height: u32 },
    Audio { duration_ms: u32 },
    Failed,
}

pub struct AssetReady {
    pub id: u32,
    pub meta: AssetMeta,
}

pub enum AssetState {
    Loading,
    Ready(AssetMeta),
    Failed,
}

pub struct AssetServer {
    next_id: Arc<AtomicU32>,
    interned: Arc<RwLock<FastHashMap<Arc<str>, u32>>>,
    states: FastHashMap<u32, AssetState>,
    requests: Sender<AssetRequest>,
    ready: Receiver<AssetReady>,
}
