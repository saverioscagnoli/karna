use macros::Get;
use std::time::{Duration, Instant};

#[derive(Debug)]
#[derive(Get)]
pub struct Time {
    pub(crate) this_frame: Instant,
    pub(crate) last_frame: Instant,

    #[get(copied, name = "frame")]
    /// The duration of the last frame
    pub(crate) frame_time: Duration,

    #[get(copied, name = "elapsed")]
    elapsed_time: Duration,
    #[get(copied, name = "delta")]
    delta_time: f32,

    #[get(pre = round, cast = u32)]
    fps: f32,
    #[get(copied)]
    tps: u32,

    frame_step: Duration,
    tick_step: Duration,
    tick_accumulator: f32,

    tick_counter: u32,
    tick_timer: f32,

    #[get(copied, name = "tick")]
    /// The duration of the last tick
    tick_time: Duration,
}

impl Time {
    const FPS_SMOOTHING: f32 = 0.1;

    pub(crate) fn new() -> Self {
        Self {
            this_frame: Instant::now(),
            last_frame: Instant::now(),
            frame_time: Duration::ZERO,
            elapsed_time: Duration::ZERO,
            delta_time: 0.0,
            fps: 0.0,
            tps: 0,
            frame_step: Duration::from_secs_f32(1.0 / 60.0),
            tick_step: Duration::from_secs_f32(1.0 / 60.0),
            tick_accumulator: 0.0,
            tick_counter: 0,
            tick_timer: 0.0,
            tick_time: Duration::ZERO,
        }
    }

    #[inline]
    pub(crate) fn frame_start(&mut self) {
        self.this_frame = Instant::now()
    }

    #[inline]
    pub(crate) fn update(&mut self) {
        let dt = self.this_frame - self.last_frame;
        self.elapsed_time += dt;

        let dt = dt.as_secs_f32();

        self.delta_time = dt;
        self.tick_timer += dt;
        self.tick_accumulator += dt;

        self.fps = self.fps * (1.0 - Self::FPS_SMOOTHING) + (1.0 / dt) * Self::FPS_SMOOTHING;

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
    /// Performs a tick and decreases the fixed time accumulator
    pub(crate) fn do_tick(&mut self, update_start: Instant) {
        self.tick_accumulator -= self.tick_step.as_secs_f32();
        self.tick_counter += 1;
        self.tick_time = Instant::now() - update_start;
    }

    #[inline]
    pub fn frame_end(&mut self) {
        self.last_frame = self.this_frame;
        self.frame_time = Instant::now() - self.this_frame;
    }

    #[inline]
    /// Calculates how much time the loop should sleep
    /// until the next frame
    pub(crate) fn until_next_frame(&self) -> Duration {
        self.frame_step.saturating_sub(self.frame_time)
    }
}
