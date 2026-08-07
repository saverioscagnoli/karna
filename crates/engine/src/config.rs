use std::sync::OnceLock;

use logging::debug;

#[derive(Debug, Clone)]
pub struct DefaultConfig {
    pub window_title: String,
    pub window_size: math::Size<u32>,
    pub target_fps: u32,
    pub target_tps: u32,
    pub max_tick_catchup: u32,
}

impl Default for DefaultConfig {
    fn default() -> Self {
        Self {
            window_title: String::from("My Window"),
            window_size: math::Size::new(800, 600),
            target_fps: 60,
            target_tps: 60,
            max_tick_catchup: 5,
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
