use std::mem;

use logging::error;
use utils::FastHashMap;

use crate::SceneId;
use crate::assets::AssetServer;
use crate::clock::Clock;
use crate::render::World;
use crate::scene::BoxedScene;
use crate::scene::SceneBuilder;
use crate::window::Window;
use crate::window::context::ForContext;
use crate::window::context::ForContextMut;
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
    pub world: World,
}

impl WindowState {
    pub fn sync_window(&mut self, window: &Window) {
        self.ctx.window.sync(&window);
        self.ctx.window.roll_mouse();
    }

    pub fn sync_time(&mut self, clock: &Clock) {
        self.ctx.time.sync(clock, &self.pacer);
    }

    pub fn load_active_scenes<'a>(&mut self, mut fctx: ForContextMut<'a>) {
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

            let ctx = ctx.for_load(&mut fctx);
            let scene = builder(ctx);

            scenes.insert(*id, SceneSlot::Loaded { scene });
        }
    }

    pub fn update_active_scenes<'a>(&mut self, phase: UpdatePhase, mut fctx: ForContextMut<'a>) {
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

            let ctx = ctx.for_update(&mut fctx);

            match phase {
                UpdatePhase::Fixed => scene.fixed_update(ctx),
                UpdatePhase::Unrestrained => scene.update(ctx),
            }
        }
    }

    pub fn render(&mut self, window: &Window, assets: &AssetServer) {
        self.world
            .render(window, assets, self.ctx.window.clear_color);
    }

    pub fn draw_active_scenes<'a>(&mut self, fctx: ForContext<'a>) {
        #[rustfmt::skip]
        let Self { ctx, scenes, scenes_active, world, .. } = self;

        world.begin_frame(ctx.window.size());

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

            let ctx = ctx.for_draw(&fctx);
            let mut draw = world.draw(ctx.window.size(), fctx.assets);

            scene.draw(ctx, &mut draw);
        }
    }
}
