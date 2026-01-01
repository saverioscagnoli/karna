//! Default logger implementation.

use crate::{
    Config, LogLevel, Logger, Record,
    formatter::{DefaultFormatter, Formatter},
};

pub struct DefaultLogger {
    config: Config,
}

unsafe impl Send for DefaultLogger {}
unsafe impl Sync for DefaultLogger {}

impl DefaultLogger {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl Logger for DefaultLogger {
    fn enabled(&self, level: LogLevel) -> bool {
        level >= self.config.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.level) {
            return;
        }

        for target_config in self.config.targets.iter() {
            let formatted = match &target_config.formatter {
                Some(f) => f.format(record),
                None => DefaultFormatter.format(record),
            };

            if let Err(e) = target_config.target.write(record.level, &formatted) {
                eprintln!("Could't write to target: {}", e)
            }
        }
    }

    fn flush(&mut self) {}
}
