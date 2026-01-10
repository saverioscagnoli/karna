use globals::{consts, profiling};
use logging::info;
use macros::{Get, Set};
use spin_sleep::SpinSleeper;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use utils::Timer;

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

    // :3
    #[get(copied, pre = round, cast = u32)]
    /// Average frames per second using exponential smoothing
    fps: f32,
    #[get(copied, name = "fps_uncapped")]
    fps_uncap: bool,

    /// FPS calculation stuff
    frame_times: VecDeque<Duration>,
    frame_times_sum: Duration,
    fps_sample_size: usize,

    #[get(copied)]
    /// Average ticks per second
    tps: u32,
    tick_counter: u32,
    tick_timer: Timer,
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
            frame_times: VecDeque::new(),
            frame_times_sum: Duration::ZERO,
            fps_sample_size: 80,
            tps: 0,
            tick_counter: 0,
            tick_timer: Timer::new(Duration::from_secs(1)),
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
    /// Marks the start of the frame
    #[inline]
    pub(crate) fn frame_start(&mut self) {
        self.this_frame = Instant::now();
    }

    /// Updates the time state
    /// calculates fps
    /// calculates tps
    #[inline]
    pub(crate) fn update(&mut self) {
        let dt = self.this_frame - self.last_frame;
        let dtf = dt.as_secs_f32();

        // Prevent spiral of death: never simulate more than 5-10 frames worth of time per frame
        // If the game lags more than this, it enters "slow motion" rather than freezing.
        let max_dt = 1.0 / 10.0;
        let raw_dt = dtf.min(max_dt);

        // Update timers
        self.elapsed_time += dt;
        self.tick_timer.tick(dtf);

        self.delta_time = (consts::DELTA_SMOOTHING * raw_dt
            + (1.0 - consts::DELTA_SMOOTHING) * self.delta_time)
            * self.time_scale;
        self.tick_accumulator += raw_dt;

        self.frame_times.push_back(dt);
        self.frame_times_sum += dt;

        if self.frame_times.len() > self.fps_sample_size
            && let Some(old_frame) = self.frame_times.pop_front()
        {
            self.frame_times_sum -= old_frame;
        }

        let avg_fps = self.frame_times_sum.as_secs_f32() / self.frame_times.len() as f32;

        self.fps = if avg_fps > f32::EPSILON {
            (1.0 / avg_fps) as f32
        } else {
            0.0
        };

        profiling::update_time(self.delta_time, self.fps, self.tps);

        if self.tick_timer.is_finished() {
            self.tps = self.tick_counter;
            self.tick_counter = 0;
            self.tick_timer.reset();
        }

        self.last_frame = self.this_frame;
    }

    /// Checks whether the game loop should do a tick or not
    ///
    /// If yes, it returns the instant at which the tick started,
    /// used to calculate how much time did the tick take
    #[inline]
    pub(crate) fn next_tick(&self) -> Option<Instant> {
        match self.tick_accumulator >= self.tick_step_f32 {
            true => Some(Instant::now()),
            false => None,
        }
    }

    /// Performs a tick
    #[inline]
    pub(crate) fn do_tick(&mut self, update_start: Instant) {
        self.tick_accumulator -= self.tick_step_f32;
        self.tick_counter += 1;
        self.tick_time = Instant::now() - update_start;
    }

    /// Marks the end of the frame
    #[inline]
    pub(crate) fn frame_end(&mut self) {
        self.frame_time = Instant::now() - self.this_frame;
    }

    /// Blocks until next frame
    /// Basically, if the target frame rate - the frame time is > 0,
    /// it means that the frame was completed before the next frame was supposed to start
    /// so the thread should sleep for this duration
    #[inline]
    pub(crate) fn wait_for_next_frame(&self) {
        if self.fps_uncap {
            return;
        }

        let elapsed = self.this_frame.elapsed();

        // Add a little bit of headroom to hit fps target more consistently
        // This should make it up for timing precision, os scheduling, and various noise
        let target = self.frame_step.saturating_sub(Duration::from_micros(25));
        let remaining = target.saturating_sub(elapsed);

        self.sleeper.sleep(remaining);
    }

    /// Sets the target frame rate
    ///
    /// NOTE: This is not guaranteed.
    /// If the game is too heavy, the machine could not be able
    /// to keep up with the game loop and run at a lower framerate.
    #[inline]
    pub fn set_target_fps(&mut self, target: u32) {
        let step = 1.0 / target as f32;

        info!("Setting target fps to {}. frame step: {}", target, step);

        self.fps_uncap = false;
        self.frame_step = Duration::from_secs_f32(step);
        self.frame_step_f32 = step;
    }

    /// Uncaps the frame rame, making the game run
    /// as fast as possible.
    ///
    /// Not recommended since in consumes a lot of cpu resources for nothing,
    /// since you can percieve the frame rate only for your specific monitor's
    /// refresh rate
    #[inline]
    pub fn uncap_fps(&mut self) {
        self.fps_uncap = true;
    }

    /// Sets the target ticks per second
    ///
    /// NOTE: the game loop is **Guaranteed** to perform
    /// at least target ticks per frame, even if that
    /// slows down the rendering.
    #[inline]
    pub fn set_target_tps(&mut self, target: u32) {
        let step = 1.0 / target as f32;

        info!("Setting target tps to {}. tick step: {}", target, step);

        self.tick_step = Duration::from_secs_f32(step);
        self.tick_step_f32 = step;
    }

    /// Returns the interpolation factor (alpha) for rendering.
    /// How much is left in the accumulator divided by the step size
    /// 0.0 = state at the previous tick
    /// 1.0 = state at the current tick
    #[inline]
    pub fn alpha(&self) -> f32 {
        self.tick_accumulator / self.tick_step_f32
    }
}
