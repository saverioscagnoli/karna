use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub enum KarnaError {
    /// (Trying to do what?, actual error)
    Sdl(String, String),
    /// Gl error, message
    OpenGL(u32, String),

    /// Message
    LoadResource(String),
}

impl Display for KarnaError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KarnaError::Sdl(where_,error) => write!(f, "SDL error in {}: {}; This probably is an internal error, please submit an issue to https://github.com/saverioscagnoli/karna.", where_, error),
            KarnaError::OpenGL(code, message) => write!(f, "OpenGL error ({}): {}; This probably is an internal error, please submit an issue to https://github.com/saverioscagnoli/karna.", code, message),
            KarnaError::LoadResource(message) => write!(f, "Error loading resource: {}", message),
        }
    }
}

impl Error for KarnaError {}
