use core::hash::Hash;
use core::hash::Hasher;
use core::panic;

use nostd::alloc::vec::Vec;
use nostd::collections::FxHasher;
use nostd::collections::Handle;
use nostd::collections::HashMap;
use nostd::collections::SlotMap;
use nostd::path::Path;
use nostd::path::PathBuf;
use sdl3::audio::AudioSpec;
use sdl3::mixer;
use traccia::error;

use crate::assets::AssetKind;
use crate::assets::AssetQueue;
use crate::assets::AssetRequest;
use crate::assets::AssetSlot;
use crate::assets::AssetSource;

#[derive(Debug, Clone, Copy)]
pub enum AudioKind {
    Oneshot,
    Streaming,
}

#[derive(Debug, Clone)]
pub struct Audio {
    pub(crate) kind: AudioKind,
    pub(crate) spec: AudioSpec,
    pub(crate) pcm: Vec<i16>,
}

impl Audio {
    pub(crate) fn new(kind: AudioKind, spec: AudioSpec, pcm: Vec<i16>) -> Self {
        Self { kind, spec, pcm }
    }
}

#[derive(Default)]
pub struct AudioRegistry {
    pub slots: SlotMap<AssetSlot<Audio>>,
    pub paths: HashMap<PathBuf, Handle<Audio>>,
    pub bytes: HashMap<u64, Handle<Audio>>,
    pub fallback: Handle<Audio>,
}

impl AudioRegistry {
    pub const SILENCE_BYTES: &[u8] = include_bytes!("../../../../assets/silence.wav");

    pub fn load_path<P>(
        &mut self,
        path: P,
        kind: AudioKind,
        queue: &mut AssetQueue,
    ) -> Handle<Audio>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();

        if let Some(handle) = self.paths.get(&path) {
            return *handle;
        }

        let handle: Handle<Audio> = self.slots.insert(AssetSlot::Pending).cast();
        self.paths.insert(path.clone(), handle);

        let request = AssetRequest {
            slot: handle.cast(),
            source: AssetSource::Path(path),
            kind: AssetKind::Audio(kind),
        };

        if queue.submit(request).is_err() {
            error!("Asset workers are gone, cannot load more assets.");
            self.slots[handle.cast()] = AssetSlot::Failed("Asset worker stopped.".into());
        }

        handle
    }

    pub fn load_bytes(
        &mut self,
        bytes: Vec<u8>,
        kind: AudioKind,
        queue: &mut AssetQueue,
    ) -> Handle<Audio> {
        let mut hasher = FxHasher::default();
        bytes.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(handle) = self.bytes.get(&hash) {
            return *handle;
        }

        let handle: Handle<Audio> = self.slots.insert(AssetSlot::Pending).cast();
        self.bytes.insert(hash, handle);

        let request = AssetRequest {
            slot: handle.cast(),
            source: AssetSource::Bytes(bytes),
            kind: AssetKind::Audio(kind),
        };

        if queue.submit(request).is_err() {
            error!("Asset workers are gone, cannot load more assets.");
            self.slots[handle.cast()] = AssetSlot::Failed("Asset worker stopped.".into());
        }

        handle
    }

    pub fn bake(&mut self, bytes: Vec<u8>, kind: AudioKind) -> Handle<Audio> {
        let Ok(mut decoder) = mixer::Decoder::from_bytes(bytes) else {
            return Handle::INVALID;
        };

        let Ok(pcm) = decoder.decode_all::<i16>() else {
            return Handle::INVALID;
        };

        let spec = decoder.spec::<i16>();

        self.slots
            .insert(AssetSlot::Ready(Audio { kind, spec, pcm }))
            .cast()
    }

    pub fn get(&self, handle: Handle<Audio>) -> &Audio {
        let slot = self.slots.get(handle.cast()).unwrap_or_else(|| {
            self.slots
                .get(self.fallback.cast())
                .expect("No audio fallback")
        });

        match slot {
            AssetSlot::Ready(audio) => audio,
            _ => todo!("failed to get audio"),
        }
    }
}
