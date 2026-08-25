pub mod atlas;
pub mod image;
pub mod workers;

use std::path::PathBuf;

use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use logging::fatal;
use logging::warn;
use sdl3::SDL_GetBasePath;
use utils::Handle;
use utils::SlotMap;
use utils::cstr_to_pathbuf;

use crate::assets::image::DecodedImage;
use crate::assets::image::Image;
use crate::err::SDL_LastError;
use crate::gpu::texture::Filter;

pub enum AssetKind {
    Image(Filter),
    Audio,
}

pub enum AssetData {
    Image(DecodedImage),
}

pub enum AssetSource {
    Path(PathBuf),
    Bytes(Vec<u8>),
}

pub struct AssetRequest {
    slot: Handle<()>,
    source: AssetSource,
    kind: AssetKind,
}

pub struct AssetResponse {
    slot: Handle<()>,
    kind: AssetKind,
    data: Result<AssetData, String>,
}

pub enum AssetSlot<T> {
    Pending,
    Ready(T),
    Failed,
}

pub struct AssetServer {
    requests: Sender<AssetRequest>,
    responses: Receiver<AssetResponse>,
    images: SlotMap<AssetSlot<Image>>,
}

impl AssetServer {
    fn new(requests: Sender<AssetRequest>, responses: Receiver<AssetResponse>) -> Self {
        let mut this = Self {
            requests,
            responses,
            images: SlotMap::new(),
        };

        this
    }
}

fn sdl_base_path() -> Result<PathBuf, String> {
    let raw = unsafe { SDL_GetBasePath() };
    if raw.is_null() {
        return Err(SDL_LastError());
    }
    Ok(unsafe { cstr_to_pathbuf(raw) })
}

pub fn resolve_base_path() -> PathBuf {
    match sdl_base_path() {
        Ok(p) => return p,
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
