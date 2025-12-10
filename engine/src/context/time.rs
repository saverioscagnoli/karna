use macros::Get;
use std::time::{Duration, Instant};

const FPS_SAMPLE_COUNT: usize = 60;

#[derive(Debug)]
#[derive(Get)]
pub struct Time {
    this_frame: Instant,
    last_frame: Instant,

    #[get(copied, name = "elapsed")]
    elapsed_time: Duration,
    #[get(copied, name = "delta")]
    delta_time: f32,
    #[get(copied, name = "frame")]
    frame_time: Duration,
    #[get(copied, name = "tick")]
    tick_time: Duration,

    #[get(copied)]
    recommended_fps: u32,
    #[get(copied, pre = round, cast = u32)]
    fps: f32,
    #[get(copied)]
    tps: u32,

    frame_step: Duration,
    tick_step: Duration,

    frame_timer: f32,
    /// Used to calculate the fps each x seconds
    fps_interval: f32,
    // Ring buffer for FPS calculation
    frame_times: [f32; FPS_SAMPLE_COUNT],
    frame_time_index: usize,
    frame_count: usize,

    tick_accumulator: f32,
    tick_timer: f32,
    tick_counter: u32,
}

impl Time {
    pub(crate) fn new(recommended_fps: u32) -> Self {
        Self {
            this_frame: Instant::now(),
            last_frame: Instant::now(),
            elapsed_time: Duration::ZERO,
            delta_time: 0.0,
            frame_time: Duration::ZERO,
            tick_time: Duration::ZERO,
            recommended_fps,
            fps: 0.0,
            tps: 0,
            frame_step: Duration::from_secs_f32(1.0 / 60.0),
            tick_step: Duration::from_secs_f32(1.0 / 60.0),
            frame_timer: 0.0,
            fps_interval: Duration::from_millis(500).as_secs_f32(),
            frame_times: [0.0; FPS_SAMPLE_COUNT],
            frame_time_index: 0,
            frame_count: 0,
            tick_accumulator: 0.0,
            tick_timer: 0.0,
            tick_counter: 0,
        }
    }

    #[inline]
    /// Must be called at the start of each frame
    pub(crate) fn frame_start(&mut self) {
        self.this_frame = Instant::now();
    }

    #[inline]
    /// Updates the time state, calculates fps
    /// And resets the tick counter if a second has passed
    /// from the last time the ticks were measured
    pub(crate) fn update(&mut self) {
        let dt = self.this_frame - self.last_frame;

        self.elapsed_time += dt;

        let dt = dt.as_secs_f32();

        self.frame_timer += dt;
        self.tick_timer += dt;
        self.tick_accumulator += dt;
        self.delta_time = dt;

        self.frame_times[self.frame_time_index] = dt;
        self.frame_time_index = (self.frame_time_index + 1) % FPS_SAMPLE_COUNT;
        self.frame_count = (self.frame_count + 1).min(FPS_SAMPLE_COUNT);

        if self.frame_timer >= self.fps_interval {
            let sum: f32 = self.frame_times[..self.frame_count].iter().sum();
            let avg_frame_time = sum / self.frame_count as f32;
            self.fps = if avg_frame_time > 0.0 {
                1.0 / avg_frame_time
            } else {
                0.0
            };
        }

        if self.tick_timer >= 1.0 {
            self.tps = self.tick_counter;
            self.tick_timer = 0.0;
            self.tick_counter = 0;
        }
    }

    #[inline]
    /// Checks whether the game loop should do a tick or not
    ///
    /// If yes, it returns the instant at which the tick started,
    /// used to calculate how much time did the tick take
    pub(crate) fn should_tick(&self) -> Option<Instant> {
        match self.tick_accumulator >= self.tick_step.as_secs_f32() {
            true => Some(Instant::now()),
            false => None,
        }
    }

    #[inline]
    /// Performs a tick
    pub(crate) fn do_tick(&mut self, update_start: Instant) {
        self.tick_accumulator -= self.tick_step.as_secs_f32();
        self.tick_counter += 1;
        self.tick_time = Instant::now() - update_start;
    }

    #[inline]
    /// Must be called at the end of each frame
    pub(crate) fn frame_end(&mut self) {
        self.frame_time = Instant::now() - self.this_frame;
        self.last_frame = self.this_frame;
    }

    #[inline]
    /// Checks how much time is left until the next frame
    /// Basically, if the target frame rate - the frame time is > 0,
    /// it means that the frame was completed before the next frame was supposed to start
    /// so the thread should sleep for this duration
    pub(crate) fn until_next_frame(&self) -> Duration {
        self.frame_step.saturating_sub(self.frame_time)
    }

    #[inline]
    /// Sets the target frame rate
    pub fn set_target_fps(&mut self, target: u32) {
        self.frame_step = Duration::from_secs_f32(1.0 / target as f32);
    }

    #[inline]
    /// Sets the target frame rate to the
    /// recommended monitor found at startup
    pub fn set_recommended_fps(&mut self) {
        self.frame_step = Duration::from_secs_f32(1.0 / self.recommended_fps as f32);
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
        self.tick_step = Duration::from_secs_f32(1.0 / target as f32);
    }
}
