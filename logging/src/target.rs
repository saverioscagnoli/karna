//! Target module
//!
//! Contains the trait that one must implement to create a custom target.
//! The crate provides 2 default targets: Console and File.

use crate::{
    Color, Colorize, LogLevel, Record, err::LogError, format_context, formatter::Formatter,
};
use std::sync::atomic::{AtomicUsize, Ordering};
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

#[derive(Default)]
pub struct DefaultConsoleFormatter;

static MAX_CONTEXT_WIDTH: AtomicUsize = AtomicUsize::new(0);

impl Formatter for DefaultConsoleFormatter {
    fn format(&self, record: &Record) -> String {
        let level_str = record.level.to_string();
        let level = format!(
            "[{}]{:<padding$}",
            level_str,
            "",
            padding = LogLevel::MAX_WIDTH - level_str.len()
        )
        .color(record.level.console_color());

        let ctx = format_context(&record.context);

        if ctx.is_empty() {
            format!("{} {}", level, record.message)
        } else {
            let ctx_len = ctx.len();
            let mut current_max = MAX_CONTEXT_WIDTH.load(Ordering::Relaxed);
            while ctx_len > current_max {
                match MAX_CONTEXT_WIDTH.compare_exchange(
                    current_max,
                    ctx_len,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => current_max = x,
                }
            }

            let max_width = MAX_CONTEXT_WIDTH.load(Ordering::Relaxed).max(30);
            let ctx_formatted =
                format!("{:<width$}", ctx, width = max_width).color(Color::RGB(128, 128, 128));

            format!("{} {} | {}", level, ctx_formatted, record.message)
        }
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
