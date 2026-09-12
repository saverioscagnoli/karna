use core::time::Duration;

use math::SdlFloat;
use sdl3::window::WindowId;

use crate::event::AppEvent;
use crate::event::Outbox;
use crate::event::WindowEvent;
use crate::time::Clock;
use crate::time::FpsCalculationStrategy;
use crate::time::FramePacer;

pub struct TimeData {
    delta: f32,
    fixed_delta: f32,
    fps: f32,
    fps_calculation_strategy: FpsCalculationStrategy,
    frame: Duration,
    alpha: f32,
}

impl Default for TimeData {
    fn default() -> Self {
        Self {
            delta: 0.0,
            fixed_delta: 0.0,
            fps: 0.0,
            fps_calculation_strategy: FpsCalculationStrategy::default(),
            frame: Duration::ZERO,
            alpha: 0.0,
        }
    }
}

impl TimeData {
    pub(crate) fn sync(&mut self, clock: &Clock, pacer: &FramePacer) {
        self.delta = pacer.delta.as_secs_f32();
        self.fixed_delta = clock.tick_rate.as_secs_f32();
        self.fps = pacer.counter.fps();
        self.fps_calculation_strategy = pacer.counter.strategy;
        self.frame = pacer.counter.average_frame_time().unwrap_or(Duration::ZERO);
        self.alpha = clock.alpha();
    }
}

pub struct Time<'a> {
    pub(crate) window_id: WindowId,
    pub(crate) data: &'a TimeData,
    pub(crate) outbox: &'a mut Outbox<AppEvent>,
}

impl<'a> Time<'a> {
    pub fn delta(&self) -> f32 {
        self.data.delta
    }

    pub fn fixed_delta(&self) -> f32 {
        self.data.fixed_delta
    }

    pub fn fps(&self) -> f32 {
        self.data.fps
    }

    pub fn fps_rounded(&self) -> u32 {
        self.data.fps.sdl_round() as u32
    }

    pub fn frame(&self) -> Duration {
        self.data.frame
    }

    pub fn alpha(&self) -> f32 {
        self.data.alpha
    }

    pub fn set_target_tps(&mut self, t: u32) {
        self.outbox.push(AppEvent::SetTargetTPS(t));
    }

    pub fn set_target_fps(&mut self, t: u32) {
        self.outbox.push(AppEvent::Window {
            window: self.window_id,
            wevent: WindowEvent::SetTargetFPS(t),
        });
    }

    pub fn set_fps_calculation_strategy(&mut self, strat: FpsCalculationStrategy) {
        self.outbox.push(AppEvent::Window {
            window: self.window_id,
            wevent: WindowEvent::SetFPSCalculationStrategy(strat),
        });
    }
}
