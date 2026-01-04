use globals::profiling;
use logging::info;
use macros::{Get, Set};
use spin_sleep::SpinSleeper;
use std::time::{Duration, Instant};

#[derive(Debug)]
#[derive(Get, Set)]
pub struct Time {
    /// T0, used to calculate delta time
    this_frame: Instant,

    /// T1, used to calculate delta time
    last_frame: Instant,

    #[get(copied, name = "delta")]
    /// How much time has passed between this frame and the previous one
    ///
    /// This is the only one which is not a `Duration`, only for convenience,
    /// since 99% of the times delta time is used as seconds.
    /// I think that the conveniece outweighs the inconsitency
    delta_time: f32,

    /// Change this value to perform slow-motion or similar effects
    #[get(copied, name = "scale")]
    #[set(name = "set_scale")]
    time_scale: f32,

    #[get(copied, name = "elapsed")]
    /// How much time has passed since the app has been created
    elapsed_time: Duration,

    #[get(copied, name = "frame")]
    /// How much time it took to draw the previous frame
    frame_time: Duration,

    #[get(copied, name = "tick")]
    /// How much time it took to perform the last tick
    tick_time: Duration,

    // COUNTERS
    #[get(copied, pre = round, cast = u32)]
    /// Average frames per second using exponential smoothing
    fps: f32,
    #[get(copied, name = "fps_uncapped")]
    fps_uncap: bool,

    #[get(copied)]
    /// Average ticks per second
    tps: u32,
    tick_counter: u32,
    tick_timer: f32,
    tick_accumulator: f32,

    /// 1.0 / TARGET FPS
    frame_step: Duration,
    frame_step_f32: f32,

    /// 1.0 / TARGET TPS
    tick_step: Duration,
    tick_step_f32: f32,

    sleeper: SpinSleeper,
}

impl Default for Time {
    fn default() -> Self {
        Self {
            this_frame: Instant::now(),
            last_frame: Instant::now(),
            delta_time: 0.0,
            time_scale: 1.0,
            elapsed_time: Duration::ZERO,
            frame_time: Duration::ZERO,
            tick_time: Duration::ZERO,
            fps: 0.0,
            fps_uncap: false,
            tps: 0,
            tick_counter: 0,
            tick_timer: 0.0,
            tick_accumulator: 0.0,
            frame_step: Duration::from_secs_f32(1.0 / 60.0),
            frame_step_f32: 1.0 / 60.0,
            tick_step: Duration::from_secs_f32(1.0 / 60.0),
            tick_step_f32: 1.0 / 60.0,
            sleeper: SpinSleeper::default(),
        }
    }
}

impl Time {
    const DELTA_SMOOTHING: f32 = 0.2;
    const FPS_SMOOTHING: f32 = 0.1;

    #[inline]
    /// Marks the start of the frame
    pub(crate) fn frame_start(&mut self) {
        self.this_frame = Instant::now();
    }

    #[inline]
    /// Updates the time state
    /// calculates fps
    /// calculates tps
    pub(crate) fn update(&mut self) {
        let dt = self.this_frame - self.last_frame;
        let dtf = dt.as_secs_f32();

        // Prevent spiral of death: never simulate more than 5-10 frames worth of time per frame
        // If the game lags more than this, it enters "slow motion" rather than freezing.
        let max_dt = 1.0 / 10.0;
        let raw_dt = dtf.min(max_dt);

        // Update timers
        self.elapsed_time += dt;
        self.tick_timer += dtf;

        self.delta_time = (Self::DELTA_SMOOTHING * raw_dt
            + (1.0 - Self::DELTA_SMOOTHING) * self.delta_time)
            * self.time_scale;
        self.tick_accumulator += raw_dt;

        // Calculate FPS using exponential smoothing
        let instant_fps = if dtf > f32::EPSILON { 1.0 / dtf } else { 0.0 };
        self.fps = Self::FPS_SMOOTHING * instant_fps + (1.0 - Self::FPS_SMOOTHING) * self.fps;

        profiling::update_time(self.delta_time, self.fps, self.tps);

        if self.tick_timer >= 1.0 {
            self.tps = self.tick_counter;
            self.tick_timer = 0.0;
            self.tick_counter = 0;
        }

        self.last_frame = self.this_frame;
    }

    #[inline]
    /// Checks whether the game loop should do a tick or not
    ///
    /// If yes, it returns the instant at which the tick started,
    /// used to calculate how much time did the tick take
    pub(crate) fn next_tick(&self) -> Option<Instant> {
        match self.tick_accumulator >= self.tick_step_f32 {
            true => Some(Instant::now()),
            false => None,
        }
    }

    #[inline]
    /// Performs a tick
    pub(crate) fn do_tick(&mut self, update_start: Instant) {
        self.tick_accumulator -= self.tick_step_f32;
        self.tick_counter += 1;
        self.tick_time = Instant::now() - update_start;
    }

    #[inline]
    /// Marks the end of the frame;
    pub(crate) fn frame_end(&mut self) {
        self.frame_time = Instant::now() - self.this_frame;
    }

    #[inline]
    /// Blocks until next frame
    /// Basically, if the target frame rate - the frame time is > 0,
    /// it means that the frame was completed before the next frame was supposed to start
    /// so the thread should sleep for this duration
    pub(crate) fn wait_for_next_frame(&self) {
        if !self.fps_uncap {
            self.sleeper
                .sleep(self.frame_step.saturating_sub(self.frame_time));
        }
    }

    #[inline]
    /// Sets the target frame rate
    pub fn set_target_fps(&mut self, target: u32) {
        let step = 1.0 / target as f32;

        info!("Setting target fps to {}. frame step: {}", target, step);

        self.fps_uncap = false;
        self.frame_step = Duration::from_secs_f32(step);
        self.frame_step_f32 = step;
    }

    #[inline]
    pub fn uncap_fps(&mut self) {
        self.fps_uncap = true;
    }

    #[inline]
    /// Sets the target ticks per second
    pub fn set_target_tps(&mut self, target: u32) {
        let step = 1.0 / target as f32;

        info!("Setting target tps to {}. tick step: {}", target, step);

        self.tick_step = Duration::from_secs_f32(step);
        self.tick_step_f32 = step;
    }

    #[inline]
    /// Returns the interpolation factor (alpha) for rendering.
    /// How much is left in the accumulator divided by the step size
    /// 0.0 = state at the previous tick
    /// 1.0 = state at the current tick
    pub fn alpha(&self) -> f32 {
        self.tick_accumulator / self.tick_step_f32
    }
}
