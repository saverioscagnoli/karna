use nostd::alloc::vec::Vec;
use nostd::collections::Handle;
use nostd::collections::HashMap;
use nostd::collections::SlotMap;
use nostd::path::Path;
use nostd::path::PathBuf;
use nostd::sync::Sender;
use sdl3::audio::AudioSpec;

use crate::assets::AssetKind;
use crate::assets::AssetRequest;
use crate::assets::AssetSlot;
use crate::assets::AssetSource;

#[derive(Debug, Clone, Copy)]
pub enum SoundKind {
    Oneshot,
    Streaming,
}

#[derive(Debug, Clone)]
pub struct Sound {
    kind: SoundKind,
    spec: AudioSpec,
    pcm: Vec<i16>,
}

impl Sound {
    pub(crate) fn new(kind: SoundKind, spec: AudioSpec, pcm: Vec<i16>) -> Self {
        Self { kind, spec, pcm }
    }
}

#[derive(Default)]
pub struct AudioRegistry {
    pub slots: SlotMap<AssetSlot<Sound>>,
    pub paths: HashMap<PathBuf, Handle<Sound>>,
}

impl AudioRegistry {
    pub fn load_path<P>(
        &mut self,
        path: P,
        kind: SoundKind,
        requests: Sender<AssetRequest>,
    ) -> Handle<Sound>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().to_path_buf();

        if let Some(handle) = self.paths.get(&path) {
            return *handle;
        }

        let handle: Handle<Sound> = self.slots.insert(AssetSlot::Pending).cast();
        self.paths.insert(path.clone(), handle);

        let request = AssetRequest {
            slot: handle.cast(),
            source: AssetSource::Path(path),
            kind: AssetKind::Audio(kind),
        };
    }
}
