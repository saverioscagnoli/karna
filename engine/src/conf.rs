use std::sync::LazyLock;

use sdl3::version::Version;
use sdl3::version::version;

pub struct Config {
    pub karna_verson: String,
    pub sdl_version: Version,
    pub target_fps: u32,
    pub target_tps: u32,
    pub present_mode: gpu::PresentMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            karna_verson: env!("CARGO_PKG_VERSION").to_string(),
            sdl_version: version(),
            target_fps: 60,
            target_tps: 60,
            present_mode: gpu::PresentMode::Vsync,
        }
    }
}

static CONFIG: LazyLock<Config> = LazyLock::new(Config::default);

pub fn config() -> &'static Config {
    &CONFIG
}
