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
pub trait Scene {
    fn load(&mut self, ctx: ContextRef, scene: &mut SceneRef);
    fn fixed_update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}
    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef);
    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw);
}

#[derive(Default)]
pub struct SceneRegistry {
    scenes: FastHashMap<u32, Box<dyn Scene>>,
}

impl SceneRegistry {
    pub fn insert_scene(&mut self, id: u32, scene: Box<dyn Scene>) {
        self.scenes.insert(id, scene);
    }

    pub fn get_mut(&mut self, id: &u32) -> &mut Box<dyn Scene> {
        match self.scenes.get_mut(id) {
            Some(s) => s,
            None => fatal!("There isn't any scene with id '{}'", id),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScenePhase {
    Load,
    FixedUpdate,
    Update,
}

pub struct World {
    scenes: SceneRegistry,
    active_scenes: Vec<u32>,
}

impl World {
    pub fn new(mut scenes: Vec<(u32, Box<dyn Scene>)>, active: Vec<u32>) -> Self {
        let mut reg = SceneRegistry::default();

        for (id, s) in mem::take(&mut scenes) {
            reg.insert_scene(id, s);
        }

        Self {
            scenes: reg,
            active_scenes: active,
        }
    }

    pub fn run(
        &mut self,
        phase: ScenePhase,
        context: &mut WindowContext,
        time: &mut Time,
        assets: &mut Assets,
    ) {
        for id in &self.active_scenes {
            let scene = self.scenes.get_mut(id);
            let (ctx, mut s) = context.split_scene(time, assets);

            match phase {
                ScenePhase::Load => scene.load(ctx, &mut s),
                ScenePhase::FixedUpdate => scene.fixed_update(ctx, &mut s),
                ScenePhase::Update => scene.update(ctx, &mut s),
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
        for id in &self.active_scenes {
            let scene = self.scenes.get_mut(id);
            let (ctx, mut d) = context.split_draw(time, assets, imgui);

            scene.draw(ctx, &mut d);
        }
    }
}
