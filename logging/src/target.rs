use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

use parking_lot::Mutex;

pub trait Target: Send + Sync {
    fn write(&self, level: log::Level, message: &str) -> Result<(), Box<dyn std::error::Error>>;
    fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Output {
    Stdout,
    #[default]
    Stderr,
}

#[derive(Default)]
pub struct Console {
    output: Output,
}

impl Console {
    pub fn new(output: Output) -> Self {
        Self { output }
    }
}

impl Target for Console {
    fn write(&self, _level: log::Level, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self.output {
            Output::Stdout => writeln!(io::stdout(), "{}", message)?,
            Output::Stderr => writeln!(io::stderr(), "{}", message)?,
        }

        Ok(())
    }
}

pub struct File {
    mu: Mutex<fs::File>,
}

impl File {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            mu: Mutex::new(file),
        })
    }
}

impl Target for File {
    fn write(&self, _: log::Level, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = self.mu.lock();
        let message = strip_ansi(message);

        writeln!(file, "{}", message)?;

        Ok(())
    }

    fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.mu.lock().flush()?;
        Ok(())
    }
}

/// Logs to the browser's developer console. Only available on the web.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub struct WebConsole;

#[cfg(target_arch = "wasm32")]
impl Target for WebConsole {
    fn write(&self, level: log::Level, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let message = wasm_bindgen::JsValue::from_str(&strip_ansi(message));

        match level {
            log::Level::Error => web_sys::console::error_1(&message),
            log::Level::Warn => web_sys::console::warn_1(&message),
            log::Level::Info => web_sys::console::info_1(&message),
            log::Level::Debug | log::Level::Trace => web_sys::console::debug_1(&message),
        }

        Ok(())
    }
}

pub fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();

            for c in chars.by_ref() {
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
