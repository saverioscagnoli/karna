mod style;
mod target;

use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::OnceLock;

use chrono::Local;
use log::LevelFilter;
pub use log::debug;
pub use log::error;
pub use log::info;
pub use log::log;
pub use log::trace;
pub use log::warn;

pub use crate::style::Color;
pub use crate::style::Colorize;
pub use crate::style::DefaultFormatter;
pub use crate::style::Formatter;
pub use crate::style::Style;
pub use crate::target::Console;
pub use crate::target::File;
pub use crate::target::Output;
pub use crate::target::Target;

pub struct Timestamp(chrono::DateTime<Local>);

impl Timestamp {
    pub fn now() -> Self {
        Self(chrono::Local::now())
    }
}

impl Deref for Timestamp {
    type Target = chrono::DateTime<Local>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Timestamp {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Logger {
    min_level: log::LevelFilter,
    targets: Vec<Box<dyn Target>>,
    formatter: Box<dyn Formatter>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.min_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = self.formatter.format(record, Timestamp::now());

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
    min_level: LevelFilter,
    targets: Vec<Box<dyn Target>>,
    formatter: Box<dyn Formatter>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_level: log::LevelFilter::Info,
            targets: vec![Box::new(Console::new(Output::Stderr))],
            formatter: Box::new(DefaultFormatter),
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

    fn build_logger(self) -> Logger {
        Logger {
            min_level: self.min_level,
            targets: self.targets,
            formatter: self.formatter,
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
