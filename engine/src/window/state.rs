use std::mem;

use logging::fatal;
use utils::FastHashMap;

use crate::render::scene_ref::SceneRef;
use crate::scene::Scene;
use crate::scene::SceneBuilder;
use crate::scene::SceneId;
use crate::window::context::UserContext;

pub enum SceneSlot {
    Unloaded(SceneBuilder),
    Loaded(Box<dyn Scene>),
    Poisoned,
}

pub struct WindowState {
    pub ctx: UserContext,
    pub scenes: FastHashMap<SceneId, SceneSlot>,
    pub active_scenes: Vec<SceneId>,
}

impl WindowState {
    pub fn load_one(&mut self, id: SceneId) {
        let Some(slot) = self.scenes.get_mut(&id) else {
            fatal!("No scene registered under id: {:?}", id);
        };

        // Builder function must be taken out before calling,
        // Because its a FnOnce
        let builder = match mem::replace(slot, SceneSlot::Poisoned) {
            SceneSlot::Unloaded(f) => f,
            SceneSlot::Poisoned => fatal!("Scene {:?} re-entered during load", id),
            loaded @ SceneSlot::Loaded(_) => {
                *slot = loaded;
                return;
            }
        };

        let s = &mut SceneRef {};
        let (ctx, mut scene_ref) = self.ctx.split_load(s);
        let scene = builder(ctx, &mut scene_ref);

        self.scenes.insert(id, SceneSlot::Loaded(scene));
    }

    pub fn load_active(&mut self) {
        for id in self.active_scenes.clone() {
            self.load_one(id);
        }
    }
}
