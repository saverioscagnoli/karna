use math as m;
use std::sync::OnceLock;

use logging::debug;

use crate::Color;
use crate::render::Layer;
use crate::window::FpsCalcStrategy;

#[derive(Default)]
#[derive(Debug, Clone)]
pub struct Config {
    pub window: WindowConfig,
    pub time: TimeConfig,
    pub gpu: GpuConfig,
    pub asset: AssetConfig,
    pub render: RenderConfig,
}

#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub title: String,
    pub size: m::Size<u32>,
    pub resizable: bool,
    pub clear_color: Color,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::from("New Window"),
            size: m::Size::new(800, 600),
            resizable: true,
            clear_color: Color::hex(0x252525),
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

#[derive(Debug, Clone)]
pub struct GpuConfig {
    pub debug: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self { debug: true }
    }
}

#[derive(Debug, Clone)]
pub struct AssetConfig {
    pub worker_threads: usize,
    pub atlas_padding: u32,
    pub atlas_page_size: u32,
}

impl Default for AssetConfig {
    fn default() -> Self {
        Self {
            worker_threads: 4,
            atlas_padding: 1,
            atlas_page_size: 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub draw_color: Color,
    pub layer: Layer,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            draw_color: Color::WHITE,
            layer: Layer::WORLD,
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
