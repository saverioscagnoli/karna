pub mod core;
pub mod input;
pub mod math;
pub mod perf;
pub mod render;
pub mod traits;
pub mod window;

use colored::Colorize;
use std::fmt;

#[macro_export]
macro_rules! throw {
    ($msg:expr) => {
        panic!("{}", $msg)
    };
}

#[derive(Debug)]
pub enum Error {
    Sdl(String),
    Window(String),
    Render(String),
}

impl Error {
    pub(crate) fn prefix(&self) -> String {
        let now = chrono::Local::now().to_rfc2822().yellow();

        match self {
            Error::Sdl(_) => format!("\n[{}]\n{}\n", now, "!!! SDL error !!!".red()),
            Error::Window(_) => format!("\n[{}]\n{}\n", now, "!!! Window error !!!".red()),
            Error::Render(_) => format!("\n[{}]\n{}\n", now, "!!! Render error !!!".red()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Sdl(msg) => write!(f, "{}{}\n", self.prefix(), msg),
            Error::Window(msg) => write!(f, "{}{}\n", self.prefix(), msg),
            Error::Render(msg) => write!(f, "{}{}\n", self.prefix(), msg),
        }
    }
}

impl std::error::Error for Error {}
