use std::ops::Deref;
use std::ops::DerefMut;

use crate::event::AppEvent;
use crate::scene::World;
use crate::window::clock::Clock;
use crate::window::context::WindowContext;
use crate::window::pacer::FramePacer;
use crate::window::time::TimeCommand;

pub struct WindowState {
    pub context: WindowContext,
    pub world: World,
    pub clock: Clock,
    pub pacer: FramePacer,

    event_queue: Vec<AppEvent>,
}

impl Deref for WindowState {
    type Target = WindowContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

impl DerefMut for WindowState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.context
    }
}

impl WindowState {
    pub fn new(context: WindowContext, world: World) -> Self {
        Self {
            context,
            world,
            clock: Clock::default(),
            pacer: FramePacer::default(),
            event_queue: Vec::new(),
        }
    }

    pub fn load(&mut self) {
        let Self {
            context,
            world,
            clock,
            pacer,
            event_queue,
        } = self;

        world.load(clock, pacer, context, event_queue);
    }

    pub fn tick(&mut self) {
        let Self {
            context,
            world,
            clock,
            pacer,
            event_queue,
        } = self;

        world.tick(clock, pacer, context, event_queue);
    }

    pub fn update(&mut self) {
        let Self {
            context,
            world,
            clock,
            pacer,
            event_queue,
        } = self;

        world.update(clock, pacer, context, event_queue);
    }

    pub fn draw(&mut self) {
        let Self {
            context,
            world,
            clock,
            pacer,
            event_queue,
        } = self;

        world.draw(clock, pacer, context, event_queue);
    }

    pub fn flush(&mut self) {
        for event in self.event_queue.drain(..) {
            match event {
                AppEvent::Time(tc) => match tc {
                    TimeCommand::SetTargetFps(t) => self.pacer.set_fps(t),
                    TimeCommand::SetTargetTps(t) => self.clock.set_tps(t),
                },
            }
        }
    }
}
