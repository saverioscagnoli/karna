use std::thread;
use std::time::Duration;
use std::time::Instant;

pub struct SleepTimer {
    margin: Duration,
}

impl SleepTimer {
    pub fn new() -> Self {
        Self {
            margin: measure_sleep_margin(),
        }
    }

    pub fn sleep_until(&self, target: Instant) {
        let now = Instant::now();

        if target <= now {
            return;
        }

        let coarse_until = target.saturating_duration_since(now);

        if coarse_until > self.margin {
            thread::sleep(coarse_until - self.margin);
        }

        while Instant::now() < target {
            thread::yield_now();
        }
    }

    pub fn sleep(&self, dur: Duration) {
        self.sleep_until(Instant::now() + dur);
    }
}

fn measure_sleep_margin() -> Duration {
    const SAMPLES: u32 = 20;

    let target = Duration::from_millis(1);
    let mut worst = Duration::ZERO;

    for _ in 0..SAMPLES {
        let start = Instant::now();

        thread::sleep(target);

        let overshoot = start.elapsed().saturating_sub(target);

        if overshoot > worst {
            worst = overshoot;
        }
    }

    // Add a small buffer on top of worst observed overshoot
    worst + Duration::from_micros(100)
}
