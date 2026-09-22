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
    Exception(Exception),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfMemory => f.write_str("quickjs: out of memory"),
            Self::InteriorNul => f.write_str("quickjs: string contains an interior nul byte"),
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
