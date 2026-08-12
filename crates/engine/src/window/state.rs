use std::mem;

use logging::error;
use sdl3::SDL_Event;
use sdl3::SDL_EventType;
use utils::FastHashMap;

use crate::clock::Clock;
use crate::input::Input;
use crate::render::draw::Draw;
use crate::render::stage::Stage;
use crate::scene::BoxedScene;
use crate::scene::SceneBuilder;
use crate::scene::SceneId;
use crate::window::context::UserContext;
use crate::window::pacer::FramePacer;
use crate::window::platform::PlatformWindow;
use crate::window::time::Time;

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
    pub time: Time,

    pub scenes: FastHashMap<SceneId, SceneSlot>,
    pub active_scenes: Vec<SceneId>,
}

impl WindowState {
    pub fn sync_time(&mut self, clock: &Clock) {
        self.time.sync(clock, &self.pacer);
    }

    pub fn sync_window(&mut self, window: &PlatformWindow) {
        self.ctx.window_handle.sync(window);
        self.ctx.window_handle.roll_mouse();
    }

    pub fn handle_window_event(&mut self, event: &SDL_Event) {
        self.ctx.window_handle.handle_event(event);
    }

    pub fn load_one(&mut self, id: SceneId, input: &Input) {
        #[rustfmt::skip]
        let Self { ctx, time, scenes, .. } = self;

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
        let ctx = ctx.for_load(time, input);
        let scene = builder(ctx, &mut view);

        scenes.insert(id, SceneSlot::Loaded { scene, stage });
    }

    pub fn load_active(&mut self, input: &Input) {
        for id in self.active_scenes.clone() {
            self.load_one(id, input);
        }
    }

    pub fn update(&mut self, phase: UpdatePhase, input: &Input) {
        #[rustfmt::skip]
        let Self { ctx, time, scenes, active_scenes, .. }  = self;

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
            let ctx = ctx.for_update(time, input);

            match phase {
                UpdatePhase::Fixed => scene.fixed_update(ctx, &mut view),
                UpdatePhase::Unrestrained => scene.update(ctx, &mut view),
            }
        }
    }

    pub fn draw(&mut self, input: &Input) {
        #[rustfmt::skip]
        let Self { ctx, draw, time, scenes, active_scenes, .. } = self;

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
            let ctx = ctx.for_draw(time, input);

            scene.draw(ctx, &mut view, draw);
        }
    }
}
