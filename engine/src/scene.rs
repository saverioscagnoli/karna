use core::panic;
use std::mem;

use imgui::Imgui;
use logging::fatal;
use utils::FastHashMap;

use crate::assets::Assets;
use crate::render::immediate::Draw;
use crate::render::retained::SceneRef;
use crate::window::context::ContextRef;
use crate::window::context::DrawContext;
use crate::window::context::WindowContext;
use crate::window::time::Time;

#[allow(unused)]
pub trait Scene: 'static {
    fn load(ctx: ContextRef, scene: &mut SceneRef) -> Self
    where
        Self: Sized;

    fn fixed_update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}
    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef);
    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw);
}

pub type SceneFactory = Box<dyn FnOnce(ContextRef, &mut SceneRef) -> Box<dyn Scene>>;

enum SceneSlot {
    Unloaded(SceneFactory),
    Loaded(Box<dyn Scene>),
    /// Transient: only observable if `load` panicked or re-entered.
    Poisoned,
}

#[derive(Default)]
pub struct SceneRegistry {
    scenes: FastHashMap<u32, SceneSlot>,
}

impl SceneRegistry {
    pub fn register<S: Scene>(&mut self, id: u32) {
        self.scenes.insert(
            id,
            SceneSlot::Unloaded(Box::new(|ctx, scene| Box::new(S::load(ctx, scene)))),
        );
    }

    pub fn insert(&mut self, id: u32, factory: SceneFactory) {
        if self.scenes.contains_key(&id) {
            fatal!("scene id {:?} registered twice", id);
        }

        self.scenes.insert(id, SceneSlot::Unloaded(factory));
    }

    fn loaded_mut(&mut self, id: u32) -> Option<&mut dyn Scene> {
        match self.scenes.get_mut(&id) {
            Some(SceneSlot::Loaded(s)) => Some(&mut **s),
            _ => None,
        }
    }

    fn get_mut(&mut self, id: u32) -> &mut Box<dyn Scene> {
        match self.scenes.get_mut(&id).expect("Failed to get scene") {
            SceneSlot::Loaded(scene) => scene,
            _ => panic!("df"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScenePhase {
    FixedUpdate,
    Update,
}

fn load_one(
    reg: &mut SceneRegistry,
    id: u32,
    context: &mut WindowContext,
    time: &mut Time,
    assets: &mut Assets,
) {
    let slot = match reg.scenes.get_mut(&id) {
        Some(slot) => slot,
        None => fatal!("no scene registered with id {:?}", id),
    };

    // The factory is FnOnce, so it must be moved out of the map to be called.
    let factory = match mem::replace(slot, SceneSlot::Poisoned) {
        SceneSlot::Unloaded(f) => f,

        // Loading twice is a no-op, not an error.
        loaded @ SceneSlot::Loaded(_) => {
            *slot = loaded;
            return;
        }

        SceneSlot::Poisoned => fatal!("scene {:?} re-entered during load", id),
    };

    let (ctx, mut s) = context.split_scene(time, assets);
    let scene = factory(ctx, &mut s);

    reg.scenes.insert(id, SceneSlot::Loaded(scene));
}

pub struct World {
    scenes: SceneRegistry,
    active_scenes: Vec<u32>,
}

impl World {
    pub fn new(scenes: Vec<(u32, SceneFactory)>, active: Vec<u32>) -> Self {
        let mut reg = SceneRegistry::default();

        for (id, factory) in scenes {
            reg.insert(id, factory);
        }

        Self {
            scenes: reg,
            active_scenes: active,
        }
    }

    pub fn load_active(
        &mut self,
        context: &mut WindowContext,
        time: &mut Time,
        assets: &mut Assets,
    ) {
        // Index loop: `load_one` needs `&mut self.scenes` while the ids are read.
        for i in 0..self.active_scenes.len() {
            let id = self.active_scenes[i];
            load_one(&mut self.scenes, id, context, time, assets);
        }
    }

    pub fn fixed_update(
        &mut self,
        context: &mut WindowContext,
        time: &mut Time,
        assets: &mut Assets,
    ) {
        for &id in &self.active_scenes {
            if let Some(scene) = self.scenes.loaded_mut(id) {
                let (ctx, mut s) = context.split_scene(time, assets);
                scene.fixed_update(ctx, &mut s);
            }
        }
    }

    pub fn update(&mut self, context: &mut WindowContext, time: &mut Time, assets: &mut Assets) {
        for &id in &self.active_scenes {
            if let Some(scene) = self.scenes.loaded_mut(id) {
                let (ctx, mut s) = context.split_scene(time, assets);
                scene.update(ctx, &mut s);
            }
        }
    }

    pub fn draw(
        &mut self,
        context: &mut WindowContext,
        time: &mut Time,
        assets: &mut Assets,
        imgui: &mut Imgui,
    ) {
        for &id in &self.active_scenes {
            if let Some(scene) = self.scenes.loaded_mut(id) {
                let (ctx, mut d) = context.split_draw(time, assets, imgui);
                scene.draw(ctx, &mut d);
            }
        }
    }
}
