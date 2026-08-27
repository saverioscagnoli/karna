use std::mem;

use logging::error;
use utils::FastHashMap;

use crate::SceneId;
use crate::clock::Clock;
use crate::scene::BoxedScene;
use crate::scene::SceneBuilder;
use crate::window::Window;
use crate::window::context::UserContext;
use crate::window::pacer::FramePacer;

pub enum SceneSlot {
    Unloaded(SceneBuilder),
    Loaded { scene: BoxedScene },
    Poisoned,
}

pub enum UpdatePhase {
    Fixed,
    Unrestrained,
}

pub struct WindowState {
    pub ctx: UserContext,
    pub pacer: FramePacer,
    pub scenes: FastHashMap<SceneId, SceneSlot>,
    pub scenes_active: Vec<SceneId>,
}

impl WindowState {
    pub fn sync_window(&mut self, window: &Window) {
        self.ctx.window.sync(&window);
        self.ctx.window.roll_mouse();
    }

    pub fn sync_time(&mut self, clock: &Clock) {
        self.ctx.time.sync(clock, &self.pacer);
    }

    pub fn load_active_scenes(&mut self) {
        #[rustfmt::skip]
        let Self { ctx, scenes, scenes_active, .. } = self;

        for id in scenes_active {
            let Some(slot) = scenes.get_mut(&id) else {
                error!("No scene registered under id: {:?}", id);
                return;
            };

            let builder = match mem::replace(slot, SceneSlot::Poisoned) {
                SceneSlot::Unloaded(f) => f,
                SceneSlot::Poisoned => return error!("Scene {:?} re-entered during load.", id),
                loaded @ SceneSlot::Loaded { .. } => {
                    *slot = loaded;
                    return;
                }
            };

            let ctx = ctx.for_load();
            let scene = builder(ctx);

            scenes.insert(*id, SceneSlot::Loaded { scene });
        }
    }

    pub fn update_active_scenes(&mut self, phase: UpdatePhase) {
        #[rustfmt::skip]
        let Self { ctx, scenes, scenes_active, .. } = self;

        for id in scenes_active {
            let Some(slot) = scenes.get_mut(&id) else {
                error!("No scene registered under id: {:?}", id);
                continue;
            };

            let scene = match slot {
                SceneSlot::Loaded { scene } => scene,
                _ => {
                    error!("Updating unloaded or poisoned scene: {:?}", id);
                    continue;
                }
            };

            let ctx = ctx.for_update();

            match phase {
                UpdatePhase::Fixed => scene.fixed_update(ctx),
                UpdatePhase::Unrestrained => scene.update(ctx),
            }
        }
    }

    pub fn draw_active_scenes(&mut self) {
        #[rustfmt::skip]
        let Self { ctx, scenes, scenes_active, .. } = self;

        for id in scenes_active {
            let Some(slot) = scenes.get_mut(&id) else {
                error!("No scene registered under id: {:?}", id);
                continue;
            };

            let scene = match slot {
                SceneSlot::Loaded { scene } => scene,
                _ => {
                    error!("Updating unloaded or poisoned scene: {:?}", id);
                    continue;
                }
            };

            let ctx = ctx.for_draw();

            scene.draw(ctx);
        }
    }
}
