use math as m;
use std::sync::OnceLock;

use logging::debug;

use crate::window::FpsCalcStrategy;

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub window: WindowConfig,
    pub time: TimeConfig,
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: m::Size<u32>,
    pub resizable: bool,
}

impl WindowConfig {
    pub const fn size_is_advisory() -> bool {
        cfg!(any(target_os = "ios", target_os = "android"))
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("New Window"),
            size: m::Size::new(800, 600),
            resizable: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimeConfig {
    pub target_fps: u32,
    pub fps_calc_strategy: FpsCalcStrategy,
    pub fps_sample_size: usize,
    pub fps_smoothing_tau: f32,
    pub target_tps: u32,
    pub max_tick_catchup: u32,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            target_fps: 60,
            fps_calc_strategy: FpsCalcStrategy::Mean,
            fps_sample_size: 120,
            fps_smoothing_tau: 0.2,
            target_tps: 60,
            max_tick_catchup: 5,
        }
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn config() -> &'static Config {
    CONFIG.get_or_init(|| {
        debug!("Using default configuration.");
        Config::default()
    })
}
