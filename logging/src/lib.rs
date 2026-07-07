mod style;
mod target;

use std::sync::Arc;
use std::sync::OnceLock;

pub use log::Level;
pub use log::LevelFilter;
pub use log::debug;
pub use log::error;
pub use log::info;
pub use log::log;
pub use log::trace;
pub use log::warn;
use parking_lot::RwLock;

pub use crate::style::Color;
pub use crate::style::Colorize;
pub use crate::style::DefaultFormatter;
pub use crate::style::Formatter;
pub use crate::style::Style;
pub use crate::target::Console;
pub use crate::target::File;
pub use crate::target::Output;
pub use crate::target::Target;
pub use crate::target::strip_ansi;

pub type SharedLogs = Arc<RwLock<Vec<(Level, Arc<str>)>>>;

pub struct Logger {
    min_level: log::LevelFilter,
    targets: Vec<Box<dyn Target>>,
    formatter: Box<dyn Formatter>,
    module_filters: Vec<(String, LevelFilter)>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let target = metadata.target();
        let level = self
            .module_filters
            .iter()
            .filter(|(prefix, _)| target.starts_with(prefix.as_str()))
            .max_by_key(|(prefix, _)| prefix.len())
            .map(|(_, lvl)| *lvl)
            .unwrap_or(self.min_level);

        metadata.level() <= level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = self.formatter.format(record);

        for t in &self.targets {
            _ = t.write(record.level(), &message);
        }
    }

    fn flush(&self) {
        for t in &self.targets {
            _ = t.flush();
        }
    }
}

pub struct Config {
    pub min_level: LevelFilter,
    pub targets: Vec<Box<dyn Target>>,
    pub formatter: Box<dyn Formatter>,
    pub module_filters: Vec<(String, LevelFilter)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_level: log::LevelFilter::Info,
            targets: vec![Box::new(Console::new(Output::Stderr))],
            formatter: Box::new(DefaultFormatter),
            module_filters: Vec::new(),
        }
    }
}

impl Config {
    pub fn with_min_level(mut self, level: log::LevelFilter) -> Self {
        self.min_level = level;
        self
    }

    pub fn with_targets(mut self, targets: Vec<Box<dyn Target>>) -> Self {
        self.targets = targets;
        self
    }

    pub fn with_target(mut self, target: Box<dyn Target>) -> Self {
        self.targets.push(target);
        self
    }

    pub fn with_formatter(mut self, formatter: Box<dyn Formatter>) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn with_module_filter<T: Into<String>>(mut self, module: T, level: LevelFilter) -> Self {
        self.module_filters.push((module.into(), level));
        self
    }

    fn build_logger(self) -> Logger {
        Logger {
            min_level: self.min_level,
            targets: self.targets,
            formatter: self.formatter,
            module_filters: self.module_filters,
        }
    }
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

pub fn init(config: Config) -> Result<(), log::SetLoggerError> {
    let logger = LOGGER.get_or_init(|| config.build_logger());

    log::set_logger(logger)?;
    log::set_max_level(logger.min_level);

    Ok(())
}
