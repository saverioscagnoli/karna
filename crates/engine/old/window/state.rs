use std::mem;

use logging::error;
use logging::fatal;
use utils::FastHashMap;

use crate::Draw;
use crate::clock::Clock;
use crate::event::AppEvent;
use crate::event::EventDispatcher;
use crate::event::SdlEvent;
use crate::render::stage::Stage;
use crate::scene::Scene;
use crate::scene::SceneBuilder;
use crate::scene::SceneId;
use crate::window::context::UserContext;
use crate::window::pacer::FramePacer;
use crate::window::time::Time;

pub enum SceneSlot {
    Unloaded(SceneBuilder),
    Loaded { scene: Box<dyn Scene>, stage: Stage },
    Poisoned,
}

pub enum UpdatePhase {
    FixedUpdate,
    Update,
}

pub struct WindowState {
    pub ctx: UserContext,
    pub pacer: FramePacer,
    pub draw: Draw,

    pub scenes: FastHashMap<SceneId, SceneSlot>,
    pub active_scenes: Vec<SceneId>,
    pub dispatcher: EventDispatcher<AppEvent>,
}

impl WindowState {
    pub fn load_one(&mut self, id: SceneId, clock: &Clock) {
        #[rustfmt::skip]
        let Self { ctx, pacer, scenes,  dispatcher, .. } = self;

        let Some(slot) = scenes.get_mut(&id) else {
            fatal!("No scene registered under id: {:?}", id);
        };

        // Builder function must be taken out before calling,
        // Because its a FnOnce
        let builder = match mem::replace(slot, SceneSlot::Poisoned) {
            SceneSlot::Unloaded(f) => f,
            SceneSlot::Poisoned => fatal!("Scene {:?} re-entered during load", id),
            loaded @ SceneSlot::Loaded { .. } => {
                *slot = loaded;
                return;
            }
        };

        let mut time = Time::snapshot(ctx.window.id, clock, pacer, dispatcher.clone());
        let mut stage = Stage::new();
        let ctx = self.ctx.load(&mut time);
        let mut view = stage.view();
        let scene = builder(ctx, &mut view);

        self.scenes.insert(id, SceneSlot::Loaded { scene, stage });
    }

    pub fn load_active(&mut self, clock: &Clock) {
        for id in self.active_scenes.clone() {
            self.load_one(id, clock);
        }
    }

    pub fn update(&mut self, phase: UpdatePhase, clock: &Clock) {
        #[rustfmt::skip]
        let Self { ctx, pacer, scenes, active_scenes, dispatcher , .. } = self;

        for id in active_scenes {
            let Some(scene) = scenes.get_mut(id) else {
                error!("Processing invalid scene: {:?}", id);
                continue;
            };

            let (scene, stage) = match scene {
                SceneSlot::Loaded { scene, stage, .. } => (scene, stage),
                _ => {
                    error!("Processing invalid or unloaded scene: {:?}", id);
                    continue;
                }
            };

            let mut time = Time::snapshot(ctx.window.id, clock, pacer, dispatcher.clone());
            let ctx = ctx.update(&mut time);
            let mut view = stage.view();

            match phase {
                UpdatePhase::FixedUpdate => scene.fixed_update(ctx, &mut view),
                UpdatePhase::Update => scene.update(ctx, &mut view),
            }
        }
    }

    pub fn draw(&mut self, clock: &Clock) {
        #[rustfmt::skip]
        let Self { ctx, pacer, draw, scenes, active_scenes, dispatcher } = self;

        for id in active_scenes {
            let Some(scene) = scenes.get_mut(id) else {
                error!("Processing invalid scene: {:?}", id);
                continue;
            };

            let (scene, stage) = match scene {
                SceneSlot::Loaded { scene, stage } => (scene, stage),
                _ => {
                    error!("Processing invalid or unloaded scene: {:?}", id);
                    continue;
                }
            };

            let mut time = Time::snapshot(ctx.window.id, clock, pacer, dispatcher.clone());
            let ctx = ctx.draw(&mut time);
            let mut view = stage.view();

            scene.draw(ctx, &mut view, draw);
        }
    }

    /// Events contain logic relative to the window must be processed here.
    /// For example, mouse motion. The mouse position is relative to the window,
    /// not the entire desktop.
    pub fn handle_event(&mut self, event: SdlEvent) {
        match event {
            SdlEvent::MouseMotion {
                x, y, xrel, yrel, ..
            } => {
                self.ctx.window.mouse_position += math::Vector2::new(x, y);
                self.ctx.window.mouse_delta += math::Vector2::new(xrel, yrel);
            }

            _ => {}
        }
    }
}
