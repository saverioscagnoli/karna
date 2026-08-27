use std::time::Duration;

use crate::clock::Clock;
use crate::events::UserEvent;
use crate::events::queue::EventDispatcher;
use crate::events::user::UserWindowEvent;
use crate::window::FpsCalcStrategy;
use crate::window::WindowId;
use crate::window::pacer::FramePacer;

pub struct Time {
    pub(crate) window_id: WindowId,
    pub(crate) delta: f32,
    pub(crate) fixed_delta: f32,
    pub(crate) fps: f32,
    pub(crate) frame: Duration,
    pub(crate) fps_calc_strategy: FpsCalcStrategy,
    pub(crate) alpha: f32,
    pub(crate) dispatcher: EventDispatcher<UserEvent>,
}

impl Time {
    pub(crate) fn new(window_id: WindowId, dispatcher: EventDispatcher<UserEvent>) -> Self {
        Self {
            window_id,
            delta: 0.0,
            fixed_delta: 0.0,
            fps: 0.0,
            frame: Duration::ZERO,
            fps_calc_strategy: FpsCalcStrategy::default(),
            alpha: 0.0,
            dispatcher,
        }
    }

    pub(crate) fn sync(&mut self, clock: &Clock, pacer: &FramePacer) {
        self.delta = pacer.delta.as_secs_f32();
        self.fixed_delta = clock.tick_rate.as_secs_f32();
        self.fps = pacer.counter.fps();
        self.frame = pacer.counter.avg_frame_time().unwrap_or(Duration::ZERO);
        self.fps_calc_strategy = pacer.counter.strategy;
        self.alpha = clock.alpha();
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn fixed_delta(&self) -> f32 {
        self.fixed_delta
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }

    pub fn frame(&self) -> Duration {
        self.frame
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn set_target_fps(&self, t: u32) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.window_id,
            wevent: UserWindowEvent::ChangeTargetFps(t),
        });
    }

    pub fn set_fps_count_strategy(&self, strategy: FpsCalcStrategy) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.window_id,
            wevent: UserWindowEvent::ChangeFpsCalcStrategy(strategy),
        });
    }

    pub fn set_target_tps(&self, t: u32) {
        self.dispatcher.dispatch(UserEvent::ChangeTargetTps(t));
    }
}
