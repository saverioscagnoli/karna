use std::mem;

use logging::fatal;
use utils::FastHashMap;

use crate::event::AppEvent;
use crate::render::immediate::Draw;
use crate::render::retained::SceneRef;
use crate::window::clock::Clock;
use crate::window::context::ContextRef;
use crate::window::context::WindowContext;
use crate::window::pacer::FramePacer;
use crate::window::time::Time;
use crate::window::time::TimeCommand;

#[allow(unused)]
pub trait Scene {
    fn load(&mut self, ctx: ContextRef, scene: &mut SceneRef);
    fn fixed_update(&mut self, ctx: ContextRef, scene: &mut SceneRef) {}
    fn update(&mut self, ctx: ContextRef, scene: &mut SceneRef);
    fn draw(&mut self, ctx: ContextRef, draw: &mut Draw);
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

    pub fn load(
        &mut self,
        clock: &Clock,
        pacer: &FramePacer,
        context: &mut WindowContext,
        events: &mut Vec<AppEvent>,
    ) {
        for id in &self.active_scenes {
            let scene = self.scenes.get_mut(id);
            let mut time = Time::snapshot(clock, pacer);
            let (ctx, mut s) = context.split_scene(&mut time);

            scene.load(ctx, &mut s);
            events.append(&mut time.take_commands());
        }
    }

    pub fn tick(
        &mut self,
        clock: &Clock,
        pacer: &FramePacer,
        context: &mut WindowContext,
        events: &mut Vec<AppEvent>,
    ) {
        for id in &self.active_scenes {
            let scene = self.scenes.get_mut(id);
            let mut time = Time::snapshot(clock, pacer);
            let (ctx, mut s) = context.split_scene(&mut time);

            scene.fixed_update(ctx, &mut s);
            events.append(&mut time.take_commands());
        }
    }

    pub fn update(
        &mut self,
        clock: &Clock,
        pacer: &FramePacer,
        context: &mut WindowContext,
        events: &mut Vec<AppEvent>,
    ) {
        for id in &self.active_scenes {
            let scene = self.scenes.get_mut(id);
            let mut time = Time::snapshot(clock, pacer);
            let (ctx, mut s) = context.split_scene(&mut time);

            scene.update(ctx, &mut s);
            events.append(&mut time.take_commands());
        }
    }

    pub fn draw(
        &mut self,
        clock: &Clock,
        pacer: &FramePacer,
        context: &mut WindowContext,
        events: &mut Vec<AppEvent>,
    ) {
        for id in &self.active_scenes {
            let scene = self.scenes.get_mut(id);
            let mut time = Time::snapshot(clock, pacer);
            let (ctx, mut d) = context.split_draw(&mut time);

            scene.draw(ctx, &mut d);
            events.append(&mut time.take_commands());
        }
    }
}
