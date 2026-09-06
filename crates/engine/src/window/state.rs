use logging::error;
use utils::FastHashMap;

use crate::SceneId;
use crate::assets::AssetServer;
use crate::clock::Clock;
use crate::render::Renderer;
use crate::scene::BoxedScene;
use crate::scene::SceneBuilder;
use crate::window::Window;
use crate::window::context::ForContextMut;
use crate::window::context::UserContext;
use crate::window::pacer::FramePacer;

pub struct SceneSlot {
    pub builder: SceneBuilder,
    pub scene: Option<BoxedScene>,
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
    pub renderer: Renderer,
}

impl WindowState {
    pub fn sync_window(&mut self, window: &Window) {
        self.ctx.window.sync(&window);
        self.ctx.window.roll_mouse();
    }

    pub fn sync_time(&mut self, clock: &Clock) {
        self.ctx.time.sync(clock, &self.pacer);
    }

    pub fn load_scene<'a>(&mut self, scene_id: SceneId, fctx: &mut ForContextMut<'a>) {
        let Self { ctx, scenes, .. } = self;

        let Some(slot) = scenes.get_mut(&scene_id) else {
            error!("Trying to load an invalid scene: {:?}", scene_id);
            return;
        };

        if slot.scene.is_none() {
            slot.scene = Some((slot.builder)(&mut ctx.for_load(fctx)));
        }
    }

    pub fn unload_scene<'a>(&mut self, scene_id: SceneId, fctx: &mut ForContextMut<'a>) {
        let Some(slot) = self.scenes.get_mut(&scene_id) else {
            return;
        };

        if let Some(ref mut scene) = slot.scene {
            let mut ctx = self.ctx.for_load(fctx);
            scene.unload(&mut ctx);
        }

        slot.scene = None;
    }

    pub fn activate_scene<'a>(&mut self, scene_id: SceneId, fctx: &mut ForContextMut<'a>) {
        if !self.scenes.contains_key(&scene_id) {
            error!("Trying to activate an invalid scene: {:?}", scene_id);
            return;
        }

        self.load_scene(scene_id, fctx);

        if !self.scenes_active.contains(&scene_id) {
            self.scenes_active.push(scene_id);
        }
    }

    pub fn deactivate_scene<'a>(&mut self, scene_id: SceneId) {
        self.scenes_active.retain(|id| &scene_id != id);
    }

    pub fn load_active_scenes<'a>(&mut self, fctx: &mut ForContextMut<'a>) {
        for id in self.scenes_active.clone() {
            self.load_scene(id, fctx);
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

            let Some(ref mut scene) = slot.scene else {
                continue;
            };

            let mut ctx = ctx.for_update(&mut fctx);

            match phase {
                UpdatePhase::Fixed => scene.fixed_update(&mut ctx),
                UpdatePhase::Unrestrained => scene.update(&mut ctx),
            }
        }
    }

    pub fn render(&mut self, window: &Window, assets: &AssetServer) {
        self.renderer
            .render(window, assets, self.ctx.window.clear_color);
    }

    pub fn draw_active_scenes<'a>(&mut self, fctx: ForContextMut<'a>) {
        #[rustfmt::skip]
        let Self { ctx, scenes, scenes_active, renderer, .. } = self;

        renderer.begin_frame(ctx.window.size(), fctx.assets);

        let mut draw = renderer.draw(ctx.window.size(), fctx.assets);

        for id in scenes_active {
            let Some(slot) = scenes.get_mut(&id) else {
                error!("No scene registered under id: {:?}", id);
                continue;
            };

            let Some(ref mut scene) = slot.scene else {
                continue;
            };

            let ctx = ctx.for_draw(fctx.input);

            scene.draw(ctx, &mut draw);
        }
    }
}
