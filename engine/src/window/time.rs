use std::mem;

use logging::warn;

use crate::event::AppEvent;
use crate::window::clock::Clock;
use crate::window::pacer::FramePacer;

pub enum TimeCommand {
    SetTargetFps(u32),
    SetTargetTps(u32),
}

pub struct Time {
    delta: f32,
    fixed_delta: f32,
    alpha: f32,
    elapsed: f32,
    ticks: u64,
    fps: u32,
    tps: u32,

    req_buffer: Vec<AppEvent>,
}

impl Time {
    pub fn snapshot(clock: &Clock, pacer: &FramePacer) -> Self {
        Self {
            delta: pacer.delta.as_secs_f32(),
            fixed_delta: clock.tick_rate.as_secs_f32(),
            alpha: clock.alpha(),
            elapsed: clock.elapsed.as_secs_f32(),
            ticks: clock.ticks,
            fps: pacer.fps.avg().round() as u32,
            tps: clock.tps.avg().round() as u32,
            req_buffer: Vec::new(),
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

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn fps(&self) -> u32 {
        self.fps
    }

    pub fn tps(&self) -> u32 {
        self.tps
    }

    pub fn set_target_fps(&mut self, t: u32) {
        if t == 0 {
            warn!("Cannot set target fps to 0.");
            return;
        }

        self.req_buffer
            .push(AppEvent::Time(TimeCommand::SetTargetFps(t)));
    }

    pub fn set_target_tps(&mut self, t: u32) {
        if t == 0 {
            warn!("Cannot set target tick rate to 0.");
            return;
        }

        self.req_buffer
            .push(AppEvent::Time(TimeCommand::SetTargetTps(t)));
    }

    pub(crate) fn take_commands(&mut self) -> Vec<AppEvent> {
        mem::take(&mut self.req_buffer)
    }
}
