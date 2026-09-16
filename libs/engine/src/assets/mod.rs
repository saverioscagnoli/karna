mod atlas;
mod image;
mod packer;
mod worker;

use nostd::alloc::string::String;
use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::path::PathBuf;
use nostd::sync::Receiver;
use nostd::sync::Sender;
use sdl3::gpu::Device;
use sdl3::image::DecodedImage;
use traccia::error;

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
    root: String,
    requests: Sender<AssetRequest>,
    responses: Receiver<AssetResponse>,
}

impl AssetServer {
    pub(crate) fn new<T>(
        root: T,
        requests: Sender<AssetRequest>,
        responses: Receiver<AssetResponse>,
    ) -> Self
    where
        T: Into<String>,
    {
        let mut this = Self {
            root: root.into(),
            requests,
            responses,
        };

        this
    }

    pub(crate) fn poll(&mut self, device: &Device) {
        use AssetKind::*;

        while let Ok(r) = self.responses.try_recv() {
            match (r.kind, r.data) {
                (Image, Ok(DecodedAsset::Image(dec))) => {}
                (Image, Err(e)) => {
                    error!("Failed to load image: {}", e);
                }

                _ => {}
            }
        }
    }
}
