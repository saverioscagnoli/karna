use logging::warn;
use utils::WindowId;

use crate::event::AppEvent;
use crate::event::EventDispatcher;
use crate::window::clock::Clock;
use crate::window::pacer::FramePacer;

#[derive(Debug)]
pub enum TimeCommand {
    SetTargetFps(u32),
    SetTargetTps(u32),
    SetPresentMode(gpu::PresentMode),
}

#[derive(Debug)]
pub struct TimeCommandRequest {
    pub window: WindowId,
    pub command: TimeCommand,
}

pub struct Time {
    window: WindowId,
    delta: f32,
    fixed_delta: f32,
    alpha: f32,
    elapsed: f32,
    ticks: u64,
    fps: u32,
    tps: u32,
    present_mode: gpu::PresentMode,
    events: EventDispatcher<AppEvent>,
}

impl Time {
    pub fn snapshot(
        window: WindowId,
        clock: &Clock,
        pacer: &FramePacer,
        mode: gpu::PresentMode,
        events: EventDispatcher<AppEvent>,
    ) -> Self {
        Self {
            window,
            delta: pacer.delta.as_secs_f32(),
            fixed_delta: clock.tick_rate.as_secs_f32(),
            alpha: clock.alpha(),
            elapsed: clock.elapsed.as_secs_f32(),
            ticks: clock.ticks,
            fps: pacer.fps.avg().round() as u32,
            tps: clock.tps.avg().round() as u32,
            present_mode: mode,
            events,
        }
    }

    fn push(&self, command: TimeCommand) {
        self.events.send(AppEvent::Time(TimeCommandRequest {
            window: self.window,
            command,
        }));
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

        self.push(TimeCommand::SetTargetFps(t));
    }

    pub fn set_target_tps(&mut self, t: u32) {
        if t == 0 {
            warn!("Cannot set target tick rate to 0.");
            return;
        }

        self.push(TimeCommand::SetTargetTps(t));
    }

    pub fn present_mode(&self) -> gpu::PresentMode {
        self.present_mode
    }

    pub fn set_present_mode(&mut self, mode: gpu::PresentMode) {
        self.push(TimeCommand::SetPresentMode(mode));
    }
}
