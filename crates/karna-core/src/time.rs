/// The fps field is based on an average of MAX_FRAME_SAMPLES frames.
const MAX_FRAME_SAMPLES: usize = 100;

/// The rate at which the frame times are updated, in seconds.
const FRAME_UPDATE_RATE: f32 = 0.1;

pub struct Time {
    delta: f32,
    accumulator: f32,

    fps: u32,
    frame_times: [f32; 100],
    frame_index: usize,
    frame_timer: f32,
}

impl Time {
    pub(crate) fn new() -> Self {
        Self {
            delta: 0.0,
            accumulator: 0.0,

            fps: 0,
            frame_times: [0.0; MAX_FRAME_SAMPLES],
            frame_index: 0,
            frame_timer: 0.0,
        }
    }

    /// Returns how much time has passed since the last frame, in seconds.
    #[inline]
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Returns the average frames per second, based on the last 100 frames.
    #[inline]
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// Returns how much time has passed since the application started, in seconds.
    ///
    /// This is calculated by adding the delta time of each frame,
    /// not with instant::now() and then elapsed(), as that would consume more resources.
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.accumulator
    }

    /// Actually updates all the time-related fields.
    #[inline]
    pub(crate) fn update(&mut self, delta: f32) {
        self.delta = delta;
        self.accumulator += delta;
        self.frame_timer += delta;

        self.frame_times[self.frame_index] = delta;
        self.frame_index = (self.frame_index + 1) % MAX_FRAME_SAMPLES;

        // If the timer passes 100ms, calculate the average fps.
        if self.frame_timer >= FRAME_UPDATE_RATE {
            self.frame_timer = 0.0;

            let sum = self.frame_times.iter().sum::<f32>();
            let avg = sum / MAX_FRAME_SAMPLES as f32;

            self.fps = (1.0 / avg).round() as u32;
        }
    }
}
