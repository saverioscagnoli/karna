use traccia::info;

use crate::assets::AssetServer;
use crate::audio::AudioSystem;
use crate::commands::AppOutboxes;
use crate::input::Input;
use crate::monitors::Monitors;
use crate::storage::SharedStore;

pub struct Services {
    pub(crate) outboxes: AppOutboxes,
    pub(crate) input: Input,
    pub(crate) assets: AssetServer,
    pub(crate) audio: AudioSystem,
    pub(crate) store: SharedStore,
    pub(crate) monitors: Monitors,
}

impl Services {
    pub(crate) fn new(assets: AssetServer) -> Self {
        Self {
            outboxes: AppOutboxes::new(),
            input: Input::default(),
            assets,
            audio: AudioSystem::new(),
            store: SharedStore::default(),
            monitors: Monitors::new(),
        }
    }

    pub(crate) fn drain_audio(&mut self) {
        for command in self.outboxes.audio.drain() {
            self.audio.apply(command, &self.assets);
        }
    }
}
