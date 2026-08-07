use std::time::Duration;

use crate::clock::Clock;
use crate::events::AppEvent;
use crate::events::EventDispatcher;
use crate::events::WindowEvent;
use crate::window::WindowId;
use crate::window::pacer::FramePacer;

pub struct Time {
    pub(crate) window_id: WindowId,
    pub(crate) delta: f32,
    pub(crate) fixed_delta: f32,
    pub(crate) dispatcher: EventDispatcher<AppEvent>,
}

impl Time {
    pub(crate) fn new(window_id: WindowId, dispatcher: EventDispatcher<AppEvent>) -> Self {
        Self {
            window_id,
            delta: 0.0,
            fixed_delta: 0.0,
            dispatcher,
        }
    }

    pub(crate) fn sync(&mut self, clock: &Clock, pacer: &FramePacer) {
        self.delta = pacer.delta.as_secs_f32();
        self.fixed_delta = clock.tick_rate.as_secs_f32();
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn fixed_delta(&self) -> f32 {
        self.fixed_delta
    }

    pub fn set_target_fps(&self, t: u32) {
        self.dispatcher.send(AppEvent::Window {
            id: self.window_id,
            event: WindowEvent::FpsTargetChangeRequested(t),
        });
    }
}
