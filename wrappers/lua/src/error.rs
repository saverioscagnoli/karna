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
    Type(String),
    Custom(String),
    Exception(Exception),
}

impl Error {
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    pub(crate) fn message(&self) -> &str {
        match self {
            Self::OutOfMemory => "not enough memory",
            Self::InteriorNul => "string contains an interior nul byte",
            Self::Type(msg) | Self::Custom(msg) => msg,
            Self::Exception(e) => &e.message,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => f.write_str("lua: out of memory"),
            Self::InteriorNul => f.write_str("lua: string contains an interior nul byte"),
            Self::Type(msg) => write!(f, "lua: type error: {msg}"),
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
