use std::sync::LazyLock;

use sdl3::version::Version;
use sdl3::version::version;

use crate::Color;

pub struct Config {
    pub karna_verson: String,
    pub sdl_version: Version,
    pub target_fps: u32,
    pub target_tps: u32,
    pub max_tick_catchup: u32,
    pub clear_color: Color,
    pub draw_color: Color,
    pub present_mode: gpu::PresentMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            karna_verson: env!("CARGO_PKG_VERSION").to_string(),
            sdl_version: version(),
            target_fps: 60,
            target_tps: 60,
            max_tick_catchup: 5,
            clear_color: Color::hex(0x252525),
            draw_color: Color::White,
            present_mode: gpu::PresentMode::Immediate,
        }
    }
}

static CONFIG: LazyLock<Config> = LazyLock::new(Config::default);

pub fn config() -> &'static Config {
    &CONFIG
}
