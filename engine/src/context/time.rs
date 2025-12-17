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
    /// Average frames per second based on the last 60 frames
    fps: f32,
    /// Interval at which the fps are calculated
    ///
    /// Default is 500ms
    fps_interval: f32,
    // usize so that FRAME_SAMPLES doest have to be converted to u32
    frame_counter: usize,
    frame_times: [f32; Self::FRAME_SAMPLES],
    frame_time_index: usize,
    frame_timer: f32,

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
            fps_interval: Duration::from_millis(500).as_secs_f32(),
            frame_counter: 0,
            frame_times: [0.0; Self::FRAME_SAMPLES],
            frame_time_index: 0,
            frame_timer: 0.0,
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
    const FRAME_SAMPLES: usize = 60;
    const DELTA_SMOOTHING: f32 = 0.2;

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
        self.frame_timer += dtf;
        self.tick_timer += dtf;

        self.delta_time = (Self::DELTA_SMOOTHING * raw_dt
            + (1.0 - Self::DELTA_SMOOTHING) * self.delta_time)
            * self.time_scale;
        self.tick_accumulator += raw_dt;

        // Take the frame sample
        self.frame_times[self.frame_time_index] = dtf;
        self.frame_time_index = (self.frame_time_index + 1) % Self::FRAME_SAMPLES;
        self.frame_counter = (self.frame_counter + 1).min(Self::FRAME_SAMPLES);

        // Check timers
        if self.frame_timer >= self.fps_interval {
            let sum: f32 = self.frame_times[..self.frame_counter].iter().sum();
            let avg_time = sum / self.frame_counter as f32;

            self.fps = if avg_time > f32::EPSILON {
                1.0 / avg_time
            } else {
                0.0
            };

            self.frame_timer = 0.0;
        }

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
        self.sleeper
            .sleep(self.frame_step.saturating_sub(self.frame_time));
    }

    #[inline]
    /// Sets the target frame rate
    pub fn set_target_fps(&mut self, target: u32) {
        let target = 1.0 / target as f32;

        self.frame_step = Duration::from_secs_f32(target);
        self.frame_step_f32 = target;
    }

    #[inline]
    /// Sets the interval duration at which the frames are calculated
    /// Default is 500ms, so fps value will be updated every 500ms
    pub fn set_fps_interval(&mut self, interval: Duration) {
        self.fps_interval = interval.as_secs_f32();
    }

    #[inline]
    /// Sets the target ticks per second
    pub fn set_target_tps(&mut self, target: u32) {
        let target = 1.0 / target as f32;

        self.tick_step = Duration::from_secs_f32(target);
        self.tick_step_f32 = target;
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
