use core::time::Duration;

use math::SdlFloat;
use nostd::alloc::collections::VecDeque;
use nostd::time::Instant;
use traccia::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaceMode {
    Display,
    Fixed,
}

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FpsCalculationStrategy {
    #[default]
    Mean,
    Smoothed,
}

pub struct FpsCounter {
    pub strategy: FpsCalculationStrategy,
    pub sample_size: usize,
    pub samples: VecDeque<Duration>,
    pub sum: Duration,
    pub ema: Option<f32>,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self {
            strategy: FpsCalculationStrategy::default(),
            sample_size: 120,
            samples: VecDeque::with_capacity(120),
            sum: Duration::ZERO,
            ema: None,
        }
    }
}

impl FpsCounter {
    pub fn push(&mut self, delta: Duration) {
        match self.strategy {
            FpsCalculationStrategy::Mean => {
                self.samples.push_back(delta);
                self.sum += delta;

                while self.samples.len() > self.sample_size {
                    if let Some(old) = self.samples.pop_front() {
                        self.sum -= old;
                    }
                }
            }
            FpsCalculationStrategy::Smoothed => {
                let dt = delta.as_secs_f32();
                let alpha = 1.0 - (-dt / 0.2).sdl_exp();

                self.ema = Some(match self.ema {
                    Some(p) => p + alpha * (dt - p),
                    None => dt,
                });
            }
        }
    }

    pub fn average_frame_time(&self) -> Option<Duration> {
        match self.strategy {
            FpsCalculationStrategy::Mean => {
                (!self.samples.is_empty()).then(|| self.sum / self.samples.len() as u32)
            }
            FpsCalculationStrategy::Smoothed => self.ema.map(Duration::from_secs_f32),
        }
    }

    pub fn fps(&self) -> f32 {
        match self.average_frame_time() {
            Some(d) if !d.is_zero() => 1.0 / d.as_secs_f32(),
            _ => 0.0,
        }
    }

    pub fn set_strategy(&mut self, strat: FpsCalculationStrategy) {
        self.strategy = strat;
        self.samples.clear();
        self.sum = Duration::ZERO;
        self.ema = None;
    }
}

pub struct FramePacer {
    pub mode: PaceMode,
    pub target_rate: Duration,
    pub last_frame: Instant,
    pub next_frame: Instant,
    pub delta: Duration,
    pub counter: FpsCounter,
}

impl FramePacer {
    pub fn new(mode: PaceMode) -> Self {
        let rate = Duration::from_secs_f32(1.0 / 60.0);

        Self {
            mode,
            target_rate: rate,
            last_frame: Instant::now(),
            next_frame: Instant::now() + rate,
            delta: rate,
            counter: FpsCounter::default(),
        }
    }

    pub fn set_target_fps(&mut self, t: u32) {
        debug!("Set target frame rate to {}", t);
        self.target_rate = Duration::from_secs_f32(1.0 / t as f32);
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
        self.target_rate
    }

    pub fn record(&mut self, now: Instant) {
        self.delta = now.duration_since(self.last_frame);
        self.counter.push(self.delta);
        self.last_frame = now;

        if self.mode == PaceMode::Fixed {
            self.next_frame += self.target_rate;

            // Fell behind
            if self.next_frame < now {
                self.next_frame = now + self.target_rate;
            }
        }
    }
}
