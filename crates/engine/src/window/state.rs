use std::mem;

use logging::error;
use utils::FastHashMap;

use crate::render::draw::Draw;
use crate::render::stage::Stage;
use crate::scene::BoxedScene;
use crate::scene::SceneBuilder;
use crate::scene::SceneId;
use crate::window::context::UserContext;
use crate::window::pacer::FramePacer;

pub enum SceneSlot {
    Unloaded(SceneBuilder),
    Loaded { scene: BoxedScene, stage: Stage },
    Poisoned,
}

#[derive(Debug, Clone, Copy)]
pub enum UpdatePhase {
    Fixed,
    Unrestrained,
}

pub struct WindowState {
    pub ctx: UserContext,
    pub draw: Draw,

    pub pacer: FramePacer,

    pub scenes: FastHashMap<SceneId, SceneSlot>,
    pub active_scenes: Vec<SceneId>,
}

impl WindowState {
    pub fn load_one(&mut self, id: SceneId) {
        let Self { ctx, scenes, .. } = self;

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

        let mut stage = Stage::new();
        let mut view = stage.view();
        let ctx = ctx.for_load();
        let scene = builder(ctx, &mut view);

        self.scenes.insert(id, SceneSlot::Loaded { scene, stage });
    }

    pub fn load_active(&mut self) {
        for id in self.active_scenes.clone() {
            self.load_one(id);
        }
    }

    pub fn update(&mut self, phase: UpdatePhase) {
        #[rustfmt::skip]
        let Self {ctx, scenes, active_scenes, .. }  = self;

        for id in active_scenes {
            let Some(scene) = scenes.get_mut(id) else {
                error!("No scene registered under id: {:?}", id);
                continue;
            };

            let (scene, stage) = match scene {
                SceneSlot::Loaded { scene, stage } => (scene, stage),
                _ => {
                    error!("Updating unloaded or poisoned scene: {:?}", id);
                    continue;
                }
            };

            let mut view = stage.view();
            let ctx = ctx.for_update();

            match phase {
                UpdatePhase::Fixed => scene.fixed_update(ctx, &mut view),
                UpdatePhase::Unrestrained => scene.fixed_update(ctx, &mut view),
            }
        }
    }

    pub fn draw(&mut self) {
        #[rustfmt::skip]
        let Self { ctx, draw, scenes, active_scenes, .. } = self;

        for id in active_scenes {
            let Some(scene) = scenes.get_mut(id) else {
                error!("No scene registered under id: {:?}", id);
                continue;
            };

            let (scene, stage) = match scene {
                SceneSlot::Loaded { scene, stage } => (scene, stage),
                _ => {
                    error!("Processing invalid or unloaded scene: {:?}", id);
                    continue;
                }
            };

            let mut view = stage.view();
            let ctx = ctx.for_draw();

            scene.draw(ctx, &mut view, draw);
        }
    }
}
