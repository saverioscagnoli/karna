use crate::assets::AssetServer;
use crate::commands::AppOutboxes;
use crate::input::Input;
use crate::monitors::Monitors;
use crate::storage::SharedStore;

pub struct Services {
    pub(crate) outboxes: AppOutboxes,
    pub(crate) input: Input,
    pub(crate) assets: AssetServer,
    pub(crate) store: SharedStore,
    pub(crate) monitors: Monitors,
}

impl Services {
    pub(crate) fn new(assets: AssetServer) -> Self {
        Self {
            outboxes: AppOutboxes::new(),
            input: Input::default(),
            assets,
            store: SharedStore::default(),
            monitors: Monitors::new(),
        }
    }
}
