use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum KarnaError {
    /// (Trying to do what?, actual error)
    Sdl(String, String),
}

impl Display for KarnaError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KarnaError::Sdl(where_,error) => write!(f, "SDL error in {}: {}; This probably is an internal error, please submit an issue to https://github.com/saverioscagnoli/karna.", where_, error),
        }
    }
}

impl Error for KarnaError {}
