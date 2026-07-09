use std::sync::Arc;
use std::sync::LazyLock;

use logging::SharedLogs;
use parking_lot::RwLock;

pub static LOGS: LazyLock<SharedLogs> = LazyLock::new(|| Arc::new(RwLock::new(Vec::new())));

struct EngineLogs;

impl logging::Target for EngineLogs {
    fn write(
        &self,
        level: logging::Level,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message = logging::strip_ansi(message);
        let mut lock = LOGS.write();
        lock.push((level, message.into()));

        Ok(())
    }
}

pub fn init_logging() {
    logging::init(
        logging::Config::default()
            .with_min_level(logging::LevelFilter::Debug)
            .with_module_filter("sctk", logging::LevelFilter::Error)
            .with_module_filter("naga", logging::LevelFilter::Error)
            .with_module_filter("wgpu", logging::LevelFilter::Error)
            .with_target(Box::new(EngineLogs)),
    )
    .expect("Failed to init logging");
}
