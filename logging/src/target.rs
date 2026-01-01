//! Target module
//!
//! Contains the trait that one must implement to create a custom target.
//! The crate provides 2 default targets: Console and File.

use crate::{Colorize, LogLevel, Record, err::LogError, formatter::Formatter};
use std::{io::Write, path::Path, sync::Mutex};

/// Defines an output destination for log messages.
///
/// This trait allows the logger to write formatted messages to different
/// destinations such as console, files, or custom targets.
pub trait Target {
    /// Writes a formatted log message to the target.
    ///
    /// # Arguments
    ///
    /// * `level` - The log level of the message
    /// * `formatted` - The formatted log message to write
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if the write operation failed
    fn write(&self, level: LogLevel, message: &str) -> Result<(), LogError>;
}

/// Standard console output target.
///
/// This target writes log messages to the standard output (stdout) or standard error (stderr)
/// using the Rust `println!` | `eprintln!` macro.
#[derive(Default)]
pub struct Console;

impl Console {
    pub fn new() -> Self {
        Self
    }
}

impl Target for Console {
    /// Always returns `Ok(())`.
    fn write(&self, level: LogLevel, message: &str) -> Result<(), LogError> {
        match level {
            LogLevel::Error | LogLevel::Fatal => eprintln!("{}", message),
            _ => println!("{}", message),
        }

        Ok(())
    }
}

pub struct DefaultConsoleFormatter;

impl Formatter for DefaultConsoleFormatter {
    fn format(&self, record: &Record) -> String {
        format!(
            "[{}] {}",
            record.level.to_string().color(record.level.console_color()),
            record.message
        )
    }
}

pub struct File {
    fd: Mutex<std::fs::File>,
}

impl File {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, LogError> {
        let file = std::fs::File::create(path).map_err(LogError::IoError)?;

        Ok(Self {
            fd: Mutex::new(file),
        })
    }
}

impl Target for File {
    fn write(&self, _level: LogLevel, message: &str) -> Result<(), LogError> {
        let mut file = self.fd.lock().map_err(|_| LogError::PoisonError)?;

        writeln!(file, "{}", message)?;

        Ok(())
    }
}
