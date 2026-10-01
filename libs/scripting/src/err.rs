use core::error;
use core::fmt;

#[derive(Debug)]
pub enum HostError {
    OutsideHook(&'static str),
    ReadOnly(&'static str),
    Unavailable(&'static str),
    DrawOnly,
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutsideHook(what) => write!(
                f,
                "{what} can only be used while a scene method runs, not at the top level of a script"
            ),
            Self::ReadOnly(what) => write!(f, "{what} cannot be changed during draw()"),
            Self::Unavailable(what) => write!(f, "{what} cannot be used during draw()"),
            Self::DrawOnly => f.write_str("the draw handle can only be used inside draw()"),
        }
    }
}

impl error::Error for HostError {}
