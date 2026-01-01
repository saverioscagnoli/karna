use crate::target::File;
use std::{
    fmt, io,
    sync::{MutexGuard, PoisonError},
};

#[derive(Debug)]
pub enum LogError {
    TargetWriteError(String),
    IoError(io::Error),
    PoisonError,
    NotInitialized,
    AlreadyInitialized,
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetWriteError(err) => writeln!(f, "Error while writing to target: {}", err),
            Self::IoError(err) => writeln!(f, "I/O error: {}", err),
            Self::PoisonError => writeln!(f, "Mutex lock is poisoned"),
            Self::NotInitialized => writeln!(f, "Logger not initialized"),
            Self::AlreadyInitialized => writeln!(f, "Logger already initialized"),
        }
    }
}

impl From<io::Error> for LogError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<PoisonError<MutexGuard<'_, File>>> for LogError {
    fn from(_: PoisonError<MutexGuard<'_, File>>) -> Self {
        Self::PoisonError
    }
}
