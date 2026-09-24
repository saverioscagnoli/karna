use core::error;
use core::fmt;

use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Exception {
    pub message: String,
    pub stack: Option<String>,
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)?;

        if let Some(stack) = &self.stack {
            write!(f, "\n{stack}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    OutOfMemory,
    InteriorNul,
    /// A value had the wrong type; thrown into JS as a `TypeError`.
    Type(String),
    /// A free-form error raised by Rust code; thrown into JS as an `Error`.
    Custom(String),
    Exception(Exception),
}

impl Error {
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => f.write_str("quickjs: out of memory"),
            Self::InteriorNul => f.write_str("quickjs: string contains an interior nul byte"),
            Self::Type(msg) => write!(f, "quickjs: type error: {msg}"),
            Self::Custom(msg) => f.write_str(msg),
            Self::Exception(e) => write!(f, "{e}"),
        }
    }
}

impl error::Error for Error {}

impl From<Exception> for Error {
    fn from(e: Exception) -> Self {
        Self::Exception(e)
    }
}
