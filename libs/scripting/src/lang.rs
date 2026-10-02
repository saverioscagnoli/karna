use core::fmt;

use engine::builder::WindowBuilder;
use nostd::alloc::rc::Rc;
use nostd::path::Path;

use crate::Host;
use crate::err::HostError;

#[derive(Debug, Clone, Copy)]
pub enum Hook {
    Load,
    Unload,
    FixedUpdate,
    Update,
    Draw,
}

impl Hook {
    pub fn name(self) -> &'static str {
        match self {
            Self::Load => "load",
            Self::Unload => "unload",
            Self::FixedUpdate => "fixed update",
            Self::Update => "update",
            Self::Draw => "draw",
        }
    }
}

pub trait Lang: Sized + 'static {
    type Error: fmt::Display;
    const ENTRY: &'static str;

    fn host_error(e: HostError) -> Self::Error;
    fn open(path: &Path, host: &Rc<Host<Self>>) -> Result<Self, Self::Error>;
    fn configure(&mut self, b: WindowBuilder) -> Result<WindowBuilder, Self::Error> {
        Ok(b)
    }

    fn start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn call(&mut self, hook: Hook) -> Result<(), Self::Error>;
}
