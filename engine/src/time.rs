use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

use logging::info;
use logging::warn;
use utils::SleepTimer;
use utils::profile;

pub struct Time {
    frame_step: Duration,
    next_frame: Instant,
    last_frame: Instant,
    delta_time: f32,
    smoothed_delta_time: f32,
    sleep_timer: SleepTimer,

    // Ticks (fixed updates)
    tps: u32,
    tick_step: f32,
    tick_count: u32,
    tick_acc: f32,
    tick_timer: f32,
    tick_time: Duration, // How much time the last tick took

    // FPS calculation
    fps: f32,
    target_fps: u32,
    fps_sample_size: usize,
    frame_times: VecDeque<Duration>,
    frame_times_sum: Duration,
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        let frame_step = Duration::from_secs_f32(1.0 / 60.0);

        Self {
            frame_step,
            next_frame: now + frame_step,
            last_frame: now,
            delta_time: frame_step.as_secs_f32(),
            smoothed_delta_time: frame_step.as_secs_f32(),
            sleep_timer: SleepTimer::new(),
            tps: 0,
            tick_step: 1.0 / 60.0,
            tick_count: 0,
            tick_acc: 0.0,
            tick_timer: 0.0,
            tick_time: Duration::ZERO,
            fps: 0.0,
            target_fps: 0,
            fps_sample_size: 100,
            frame_times: VecDeque::new(),
            frame_times_sum: Duration::ZERO,
        }
    }

    pub fn delta(&self) -> f32 {
        self.delta_time
    }

    pub fn smoothed_delta(&self) -> f32 {
        self.smoothed_delta_time
    }

    pub fn fixed_delta(&self) -> f32 {
        self.tick_step
    }

    pub fn tps(&self) -> u32 {
        self.tps
    }

    pub fn set_target_tps(&mut self, t: u32) {
        if t == 0 {
            warn!("Setting target tps to 0 is not allowed");
            return;
        }

        self.tick_step = 1.0 / t as f32;
    }

    pub fn tick(&self) -> Duration {
        self.tick_time
    }

    pub fn fps_sample_size(&self) -> usize {
        self.fps_sample_size
    }

    pub fn fps(&self) -> u32 {
        self.fps.round() as u32
    }

    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }

    pub fn set_target_fps(&mut self, t: u32) {
        if t == 0 {
            warn!("Setting target fps to 0 is not allowed");
            warn!("If you wanted to uncap the fps, use `time.uncap_fps()`");
            return;
        }

        self.frame_step = Duration::from_secs_f32(1.0 / t as f32);
        self.target_fps = t;
        info!("Target fps set to {}", t);
    }

    pub fn alpha(&self) -> f32 {
        self.tick_acc / self.tick_step
    }

    pub(crate) fn next_tick(&self) -> Option<Instant> {
        match self.tick_acc >= self.tick_step {
            true => Some(Instant::now()),
            false => None,
        }
    }

    pub(crate) fn do_tick(&mut self, tick_start: Instant) {
        self.tick_acc -= self.tick_step;
        self.tick_count += 1;
        self.tick_time = Instant::now() - tick_start;
        profile::record("tick", self.tick_time);
    }

    pub(crate) fn update(&mut self) {
        let now = Instant::now();

        self.next_frame += self.frame_step;

        if self.next_frame < now {
            self.next_frame = now + self.frame_step;
        }

        let dt = now.duration_since(self.last_frame);

        self.delta_time = dt.as_secs_f32().min(0.1);
        self.last_frame = now;

        const SMOOTHING: f32 = 0.1; // lower = smoother but slower to react
        self.smoothed_delta_time =
            self.smoothed_delta_time * (1.0 - SMOOTHING) + self.delta_time * SMOOTHING;

        self.frame_times.push_back(dt);
        self.frame_times_sum += dt;

        if self.frame_times.len() > self.fps_sample_size
            && let Some(old_frame) = self.frame_times.pop_front()
        {
            self.frame_times_sum -= old_frame;
        }

        let avg_frame_time = self.frame_times_sum.as_secs_f32() / self.frame_times.len() as f32;

        self.fps = if avg_frame_time > f32::EPSILON {
            1.0 / avg_frame_time
        } else {
            0.0
        };

        self.tick_acc += self.delta_time;
        self.tick_timer += self.delta_time;

        if self.tick_timer >= 1.0 {
            self.tps = self.tick_count;
            self.tick_count = 0;
            self.tick_timer = 0.0;
        }

        profile::record("frame", dt);
        profile::gauge("fps", self.fps);
        profile::gauge("tps", self.tps);
    }

    pub(crate) fn wait_for_next_frame(&mut self) {
        self.sleep_timer.sleep_until(self.next_frame);
    }
}
