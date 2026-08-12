use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

use logging::debug;

use crate::config::config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PaceMode {
    /// The display paces the frame
    Display,
    /// Pace frames manually with a spin sleeper
    Fixed,
}

#[derive(Default)]
#[derive(Debug, Clone, Copy)]
pub enum FpsCountStrategy {
    #[default]
    Mean,
    Smoothed,
}

pub struct FpsCounter {
    pub strategy: FpsCountStrategy,
    pub sample_size: usize,
    pub samples: VecDeque<Duration>,
    pub sum: Duration,
    pub ema: Option<f32>,
}

impl Default for FpsCounter {
    fn default() -> Self {
        let config = config();

        Self {
            strategy: config.fps_count_strategy,
            sample_size: config.fps_sample_size,
            samples: VecDeque::with_capacity(config.fps_sample_size),
            sum: Duration::ZERO,
            ema: None,
        }
    }
}

impl FpsCounter {
    pub fn push(&mut self, delta: Duration) {
        match self.strategy {
            FpsCountStrategy::Mean => {
                self.samples.push_back(delta);
                self.sum += delta;

                while self.samples.len() > self.sample_size {
                    if let Some(old) = self.samples.pop_front() {
                        self.sum -= old;
                    }
                }
            }

            FpsCountStrategy::Smoothed => {
                let config = config();
                let dt = delta.as_secs_f32();
                let alpha = 1.0 - (-dt / config.fps_smoothing_tau).exp();

                self.ema = Some(match self.ema {
                    Some(prev) => prev + alpha * (dt - prev),
                    None => dt,
                });
            }
        }
    }

    pub fn avg_frame_time(&self) -> Option<Duration> {
        match self.strategy {
            FpsCountStrategy::Mean => {
                (!self.samples.is_empty()).then(|| self.sum / self.samples.len() as u32)
            }

            FpsCountStrategy::Smoothed { .. } => self.ema.map(Duration::from_secs_f32),
        }
    }

    pub fn fps(&self) -> f32 {
        match self.avg_frame_time() {
            Some(d) if !d.is_zero() => 1.0 / d.as_secs_f32(),
            _ => 0.0,
        }
    }

    pub fn set_strategy(&mut self, strategy: FpsCountStrategy) {
        self.strategy = strategy;
        self.samples.clear();
        self.sum = Duration::ZERO;
        self.ema = None;
    }
}

pub struct FramePacer {
    pub mode: PaceMode,
    pub frame_rate: Duration,
    pub last_frame: Instant,
    pub next_frame: Instant,
    pub delta: Duration,
    pub counter: FpsCounter,
}

impl FramePacer {
    pub fn new(mode: PaceMode) -> Self {
        let conf = config();
        let rate = Duration::from_secs_f32(1.0 / conf.target_fps as f32);

        Self {
            mode,
            frame_rate: rate,
            last_frame: Instant::now(),
            next_frame: Instant::now(),
            delta: rate,
            counter: FpsCounter::default(),
        }
    }

    pub fn set_target_fps(&mut self, t: u32) {
        debug!("Set target fps to {}", t);
        self.frame_rate = Duration::from_secs_f32(1.0 / t as f32);
    }

    pub fn due(&self, now: Instant) -> bool {
        match self.mode {
            PaceMode::Display => true,
            PaceMode::Fixed => now >= self.next_frame,
        }
    }

    pub fn deadline(&self) -> Option<Instant> {
        match self.mode {
            PaceMode::Display => None,
            PaceMode::Fixed => Some(self.next_frame),
        }
    }

    pub fn idle_backoff(&self) -> Duration {
        self.frame_rate
    }

    pub fn record(&mut self, now: Instant) {
        self.delta = now.duration_since(self.last_frame);
        self.counter.push(self.delta);
        self.last_frame = now;

        if self.mode == PaceMode::Fixed {
            self.next_frame += self.frame_rate;

            // Fell behind
            if self.next_frame < now {
                self.next_frame = now + self.frame_rate
            }
        }
    }
}
