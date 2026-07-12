//! Global metrics registry for profiling.
//!
//! Anything, anywhere can record a metric without passing state around:
//!
//! ```
//! use utils::profile;
//!
//! profile::count("draw_calls", 1); // accumulates within the frame
//! profile::gauge("entities", 1523u32); // last value wins
//!
//! {
//!     let _guard = profile::scope("physics"); // timed until dropped
//! }
//!
//! let result = profile::time("ai", || 42); // timed closure
//! ```
//!
//! The engine calls [`end_frame`] once per frame, which rolls the current
//! values into a fixed-size history so [`snapshot`] and [`get`] can report
//! last / average / min / max over the recent past.

use std::collections::VecDeque;
use std::sync::LazyLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use parking_lot::Mutex;

use crate::FastHashMap;

/// How many completed frames each metric remembers.
const HISTORY_LEN: usize = 120;

static ENABLED: AtomicBool = AtomicBool::new(true);

static REGISTRY: LazyLock<Mutex<FastHashMap<&'static str, Metric>>> =
    LazyLock::new(|| Mutex::new(FastHashMap::default()));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Accumulates within a frame, resets to 0 at [`end_frame`].
    /// (e.g. draw calls, lock acquisitions)
    Counter,
    /// Holds the last value written, persists across frames. (e.g. fps)
    Gauge,
    /// Seconds accumulated within a frame, resets at [`end_frame`].
    /// (e.g. render time)
    Time,
}

struct Metric {
    kind: Kind,
    current: f64,
    history: VecDeque<f64>,
    sum: f64,
}

impl Metric {
    fn new(kind: Kind) -> Self {
        Self {
            kind,
            current: 0.0,
            history: VecDeque::with_capacity(HISTORY_LEN),
            sum: 0.0,
        }
    }

    fn end_frame(&mut self) {
        if self.history.len() == HISTORY_LEN
            && let Some(old) = self.history.pop_front()
        {
            self.sum -= old;
        }

        self.history.push_back(self.current);
        self.sum += self.current;

        if matches!(self.kind, Kind::Counter | Kind::Time) {
            self.current = 0.0;
        }
    }

    fn stats(&self, name: &'static str) -> Stats {
        let last = self.history.back().copied().unwrap_or(self.current);
        let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);

        for &v in &self.history {
            min = min.min(v);
            max = max.max(v);
        }

        let avg = if self.history.is_empty() {
            last
        } else {
            self.sum / self.history.len() as f64
        };

        Stats {
            name,
            kind: self.kind,
            last,
            avg,
            min: if min.is_finite() { min } else { last },
            max: if max.is_finite() { max } else { last },
        }
    }
}

/// Aggregated view of a single metric over its recent history.
///
/// [`Kind::Time`] metrics are in seconds.
#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub name: &'static str,
    pub kind: Kind,
    /// Value of the most recent completed frame.
    pub last: f64,
    pub avg: f64,
    pub min: f64,
    pub max: f64,
}

/// Globally enables or disables recording. Enabled by default.
///
/// While disabled, every recording function is a cheap no-op.
pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

fn with_metric(name: &'static str, kind: Kind, f: impl FnOnce(&mut Metric)) {
    if !enabled() {
        return;
    }

    let mut registry = REGISTRY.lock();
    let metric = registry.entry(name).or_insert_with(|| Metric::new(kind));

    f(metric);
}

/// Adds `n` to a per-frame counter. (e.g. draw calls, lock acquisitions)
pub fn count(name: &'static str, n: u32) {
    with_metric(name, Kind::Counter, |m| m.current += n as f64);
}

/// Sets a gauge to `value`. The last write in a frame wins. (e.g. fps)
pub fn gauge<V: Into<f64>>(name: &'static str, value: V) {
    let value = value.into();
    with_metric(name, Kind::Gauge, |m| m.current = value);
}

/// Adds `elapsed` to a per-frame time metric.
pub fn record(name: &'static str, elapsed: Duration) {
    with_metric(name, Kind::Time, |m| m.current += elapsed.as_secs_f64());
}

/// Times everything until the returned guard is dropped,
/// then records it like [`record`].
///
/// ```
/// let _guard = utils::profile::scope("physics");
/// ```
#[must_use = "timing stops when the guard is dropped; bind it with `let _guard = ...`"]
pub fn scope(name: &'static str) -> ScopeGuard {
    ScopeGuard {
        name,
        start: Instant::now(),
    }
}

/// Times a closure and records it like [`record`].
pub fn time<R>(name: &'static str, f: impl FnOnce() -> R) -> R {
    let _guard = scope(name);
    f()
}

pub struct ScopeGuard {
    name: &'static str,
    start: Instant,
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        record(self.name, self.start.elapsed());
    }
}

/// Rolls every metric's current value into its history and resets
/// counters and time metrics for the next frame.
///
/// The engine calls this once per frame; user code shouldn't need to.
pub fn end_frame() {
    if !enabled() {
        return;
    }

    for metric in REGISTRY.lock().values_mut() {
        metric.end_frame();
    }
}

/// Stats for a single metric, or `None` if nothing was recorded under `name`.
pub fn get(name: &str) -> Option<Stats> {
    REGISTRY
        .lock()
        .get_key_value(name)
        .map(|(name, metric)| metric.stats(name))
}

/// Stats for every registered metric, sorted by name.
pub fn snapshot() -> Vec<Stats> {
    let registry = REGISTRY.lock();
    let mut stats: Vec<Stats> = registry
        .iter()
        .map(|(name, metric)| metric.stats(name))
        .collect();

    stats.sort_by_key(|s| s.name);
    stats
}

/// Removes every registered metric.
pub fn clear() {
    REGISTRY.lock().clear();
}
