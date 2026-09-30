use core::hash::Hash;
use core::hash::Hasher;

use nostd::alloc::sync::Arc;
use nostd::alloc::vec::Vec;
use nostd::collections::FxHasher;
use nostd::collections::Handle;
use nostd::collections::HashMap;
use nostd::collections::SlotMap;
use nostd::path::Path;
use nostd::path::PathBuf;
use sdl3::SdlError;
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
pub enum AudioData {
    Oneshot(Arc<[i16]>),
    Streaming(Arc<[u8]>),
}

#[derive(Debug, Clone)]
pub struct Audio {
    pub(crate) spec: AudioSpec,
    pub(crate) data: AudioData,
}

impl Audio {
    pub(crate) fn new(spec: AudioSpec, data: AudioData) -> Self {
        Self { spec, data }
    }
}

pub(crate) fn decode_audio(
    bytes: &[u8],
    kind: AudioKind,
) -> Result<(AudioSpec, AudioData), SdlError> {
    let mut decoder = mixer::Decoder::from_bytes(bytes.to_vec())?;
    let spec = decoder.spec::<i16>();

    let data = match kind {
        AudioKind::Oneshot => AudioData::Oneshot(decoder.decode_all::<i16>()?.into()),
        AudioKind::Streaming => AudioData::Streaming(Arc::from(bytes)),
    };

    Ok((spec, data))
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
        let Ok((spec, data)) = decode_audio(&bytes, kind) else {
            return Handle::INVALID;
        };

        self.slots
            .insert(AssetSlot::Ready(Audio::new(spec, data)))
            .cast()
    }

    pub fn get(&self, handle: Handle<Audio>) -> &Audio {
        self.resolve(handle).unwrap_or_else(|| self.fallback())
    }

    pub fn resolve(&self, handle: Handle<Audio>) -> Option<&Audio> {
        match self.slots.get(handle.cast()) {
            Some(AssetSlot::Pending) => None,
            Some(AssetSlot::Ready(audio)) => Some(audio),
            _ => Some(self.fallback()),
        }
    }

    fn fallback(&self) -> &Audio {
        match self.slots.get(self.fallback.cast()) {
            Some(AssetSlot::Ready(audio)) => audio,
            _ => panic!("No audio fallback"),
        }
    }
}
