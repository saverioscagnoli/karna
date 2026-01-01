#![warn(clippy::use_self)]

pub mod target;

mod default;
mod err;
mod formatter;
mod macros;
mod style;

use crate::target::{Console, DefaultConsoleFormatter, Target};
use std::{fmt, sync::OnceLock};

// Re-exports
pub use default::DefaultLogger;
pub use err::LogError;
pub use formatter::{DefaultFormatter, Formatter};
pub use style::*;

/// Logging severity levels in ascending order of importance.
///
/// The levels follow the common convention:
/// - `Trace`: Very detailed information, typically only needed when debugging specific issues
/// - `Debug`: Detailed information useful for debugging
/// - `Info`: General information about application progress
/// - `Warn`: Potentially harmful situations that might need attention
/// - `Error`: Error events that might still allow the application to continue running
#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    Fatal,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trace => write!(f, "trace"),
            Self::Debug => write!(f, "debug"),
            Self::Info => write!(f, "info"),
            Self::Warn => write!(f, "warn"),
            Self::Error => write!(f, "error"),
            Self::Fatal => write!(f, "fatal"),
        }
    }
}

impl LogLevel {
    pub fn console_color(&self) -> Color {
        match self {
            Self::Trace => Color::Cyan,
            Self::Debug => Color::Blue,
            Self::Info => Color::Green,
            Self::Warn => Color::Yellow,
            Self::Error => Color::Red,
            Self::Fatal => Color::BrightRed,
        }
    }
}

/// Represents a single log record with all relevant metadata.
///
/// A `Record` contains the log level, target component, message content, and
/// source location information (module path, file, line).
#[derive(Debug, Clone)]
pub struct Record {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub target: String,
}

pub struct TargetConfig {
    /// Actual logging target
    pub target: Box<dyn Target>,

    /// Custom formatter for this target
    /// Will override the global formatter if set
    pub formatter: Option<Box<dyn Formatter>>,
}

/// Configuration for initializing a logger.
///
/// This struct allows customizing the logger's behavior by specifying
/// the minimum log level, output targets, and formatting options.
pub struct Config {
    /// Maximum log level target
    pub level: LogLevel,

    /// Logging targets (console, file, etc)
    pub targets: Vec<TargetConfig>,

    /// Global formatter for all targets
    pub formatter: Box<dyn Formatter>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            targets: vec![TargetConfig {
                target: Box::new(Console::new()),
                formatter: Some(Box::new(DefaultConsoleFormatter)),
            }],
            formatter: Box::new(DefaultFormatter),
        }
    }
}

impl Config {
    pub fn new(level: LogLevel, targets: Vec<TargetConfig>, formatter: Box<dyn Formatter>) -> Self {
        Self {
            level,
            targets,
            formatter,
        }
    }

    pub fn empty() -> Self {
        Self {
            level: LogLevel::Info,
            targets: Vec::new(),
            formatter: Box::new(DefaultFormatter),
        }
    }

    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_targets(mut self, targets: Vec<TargetConfig>) -> Self {
        self.targets = targets;
        self
    }

    pub fn with_target(mut self, target: TargetConfig) -> Self {
        self.targets.push(target);
        self
    }
}

/// Core trait that defines the logging behavior.
///
/// Implementors of this trait handle the actual processing and writing of log records.
/// Custom loggers can be created by implementing this trait
pub trait Logger: Send + Sync {
    /// Determines if a message with the given log level should be processed.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level to check
    ///
    /// # Returns
    ///
    /// `true` if messages at this level should be logged, `false` otherwise
    fn enabled(&self, level: LogLevel) -> bool;

    /// Process and output a log record.
    ///
    /// # Arguments
    ///
    /// * `record` - The log record to process
    fn log(&self, record: &Record);

    /// Flush any buffered log records.
    fn flush(&mut self);
}

static LOGGER: OnceLock<Box<dyn Logger>> = OnceLock::new();

pub fn logger() -> &'static dyn Logger {
    LOGGER
        .get()
        .ok_or(LogError::NotInitialized)
        .unwrap()
        .as_ref()
}

pub fn set_logger(logger: Box<dyn Logger>) {
    LOGGER
        .set(logger)
        .map_err(|_| LogError::AlreadyInitialized)
        .unwrap()
}

pub fn init(config: Config) {
    set_logger(Box::new(DefaultLogger::new(config)));
}

pub fn init_default() {
    init(Config::default());
}

pub fn init_with_level(level: LogLevel) {
    init(Config::default().with_level(level));
}

pub fn init_with_logger(logger: Box<dyn Logger>) {
    set_logger(logger);
}
