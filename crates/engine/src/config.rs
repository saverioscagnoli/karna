use std::sync::OnceLock;

use logging::debug;

use crate::gpu::present_mode::PresentMode;
use crate::render::color::Color;
use crate::window::pacer::FpsCountStrategy;

#[derive(Debug, Clone)]
pub struct DefaultConfig {
    pub window_title: String,
    pub window_size: math::Size<u32>,
    pub window_clear_color: Color,
    pub target_fps: u32,
    pub fps_count_strategy: FpsCountStrategy,
    pub fps_sample_size: usize,
    pub fps_smoothing_tau: f32,
    pub present_mode: PresentMode,
    pub target_tps: u32,
    pub max_tick_catchup: u32,
    pub draw_color: Color,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            window_title: String::from("My Window"),
            window_size: math::Size::new(1280, 720),
            window_clear_color: Color::hex(0x252525),
            target_fps: 60,
            fps_count_strategy: FpsCountStrategy::Mean,
            fps_sample_size: 100,
            fps_smoothing_tau: 0.25,
            present_mode: PresentMode::Immediate,
            target_tps: 60,
            max_tick_catchup: 5,
            draw_color: Color::White,
        }
    }
}

static CONFIG: OnceLock<DefaultConfig> = OnceLock::new();

pub fn init_config(config: Option<DefaultConfig>) {
    match config {
        Some(c) => CONFIG.set(c).expect("Config initialized already."),
        None => CONFIG
            .set(DefaultConfig::default())
            .expect("Config initialized already."),
    }

    debug!("Loaded default configuration.");
}

pub fn config() -> &'static DefaultConfig {
    CONFIG.get().expect("Config not set")
}
