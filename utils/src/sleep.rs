use std::cell::Cell;
use std::hint;
use std::thread;
use std::time::Duration;
use std::time::Instant;

/// Sleeps to a deadline by sleeping coarse and spinning the tail.
///
/// The margin between the two is measured at runtime, so it adapts to the
/// platform's timer granularity instead of relying on a hardcoded guess.
///
/// Not `Sync` — intended to be owned by a single loop.
pub struct SleepTimer {
    // Welford accumulators over observed sleep overshoot, in seconds.
    mean: Cell<f64>,
    m2: Cell<f64>,
    count: Cell<u64>,
    margin: Cell<Duration>,
}

impl SleepTimer {
    /// Cheap. Starts pessimistic and converges within a few frames.
    pub fn new() -> Self {
        Self {
            mean: Cell::new(0.0),
            m2: Cell::new(0.0),
            count: Cell::new(0),
            margin: Cell::new(Duration::from_micros(1500)),
        }
    }

    /// Spends up to ~25ms measuring the platform before returning.
    /// Call once per process, after `sdl3::init()`.
    pub fn calibrated() -> Self {
        let seed = measure_sleep_margin();
        let timer = Self::new();

        // Prime Welford with a pseudo-count: the seed carries the weight of
        // ~8 real samples, so live data dominates quickly but the first few
        // frames aren't flying blind.
        timer.mean.set(seed.as_secs_f64());
        timer.m2.set(0.0);
        timer.count.set(8);
        timer.margin.set(seed);
        timer
    }

    pub fn sleep_until(&self, target: Instant) {
        let now = Instant::now();
        if target <= now {
            return;
        }

        let remaining = target - now;
        let margin = self.margin.get();

        if remaining > margin {
            let requested = remaining - margin;
            let before = Instant::now();
            thread::sleep(requested);
            let actual = before.elapsed();
            self.observe(actual.saturating_sub(requested));
        }

        self.spin_until(target);
    }

    pub fn sleep(&self, dur: Duration) {
        self.sleep_until(Instant::now() + dur);
    }

    pub fn margin(&self) -> Duration {
        self.margin.get()
    }

    fn spin_until(&self, target: Instant) {
        loop {
            let now = Instant::now();
            if now >= target {
                return;
            }
            // Yield while there's real time left; only burn the core on the
            // last sliver, where a lost timeslice would blow the deadline.
            if target - now > Duration::from_micros(50) {
                thread::yield_now();
            } else {
                hint::spin_loop();
            }
        }
    }

    fn observe(&self, overshoot: Duration) {
        let x = overshoot.as_secs_f64();

        // Cap the window so the estimate keeps tracking a moving platform
        // rather than freezing around whatever the first few thousand frames saw.
        if self.count.get() >= 2048 {
            self.count.set(256);
            self.m2.set(self.m2.get() * 0.125);
        }

        let count = self.count.get() + 1;
        self.count.set(count);

        let delta = x - self.mean.get();
        let mean = self.mean.get() + delta / count as f64;
        self.mean.set(mean);
        self.m2.set(self.m2.get() + delta * (x - mean));

        let stddev = if count > 1 {
            (self.m2.get() / (count - 1) as f64).sqrt()
        } else {
            0.0
        };

        // mean + 1σ covers the usual case; clamp so one pathological sample
        // can't turn this into a 20ms spin.
        let est = (mean + stddev).clamp(0.0, 0.004);
        self.margin.set(Duration::from_secs_f64(est));
    }
}

impl Default for SleepTimer {
    fn default() -> Self {
        Self::new()
    }
}

fn measure_sleep_margin() -> Duration {
    const MAX_SAMPLES: usize = 16;
    const BUDGET: Duration = Duration::from_millis(25);
    let target = Duration::from_millis(1);

    // Discard one: the first sleep pays one-time costs that never recur.
    thread::sleep(target);

    let mut samples = Vec::with_capacity(MAX_SAMPLES);
    let deadline = Instant::now() + BUDGET;

    while samples.len() < MAX_SAMPLES && Instant::now() < deadline {
        let start = Instant::now();
        thread::sleep(target);
        samples.push(start.elapsed().saturating_sub(target));
    }

    if samples.is_empty() {
        return Duration::from_micros(1500);
    }

    samples.sort_unstable();
    let p75 = samples[samples.len() * 3 / 4];

    (p75 + Duration::from_micros(100)).clamp(Duration::from_micros(50), Duration::from_millis(4))
}
