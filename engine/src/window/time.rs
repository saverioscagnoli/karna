use logging::warn;

use crate::clock::Clock;
use crate::event::AppEvent;
use crate::event::EventDispatcher;
use crate::event::WindowEvent;
use crate::window::WindowId;
use crate::window::pacer::FramePacer;

pub struct Time {
    window_id: WindowId,
    delta: f32,
    fixed_delta: f32,
    alpha: f32,
    dispatcher: EventDispatcher<AppEvent>,
}

impl Time {
    pub(crate) fn snapshot(
        window_id: WindowId,
        clock: &Clock,
        pacer: &FramePacer,
        dispatcher: EventDispatcher<AppEvent>,
    ) -> Self {
        Self {
            window_id,
            delta: pacer.delta.as_secs_f32(),
            fixed_delta: clock.tick_rate.as_secs_f32(),
            alpha: clock.alpha(),
            dispatcher,
        }
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn fixed_delta(&self) -> f32 {
        self.fixed_delta
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn set_target_fps(&self, t: u32) {
        if t == 0 {
            warn!("Cannot set target fps to 0.");
            return;
        }

        self.dispatcher.send(AppEvent::Window {
            id: self.window_id,
            event: WindowEvent::FpsTargetChangeRequested(t),
        });
    }

    pub fn set_target_tps(&self, t: u32) {
        if t == 0 {
            warn!("Cannot set target tps to 0.");
            return;
        }

        self.dispatcher.send(AppEvent::TpsTargetChangeRequested(t));
    }
}
