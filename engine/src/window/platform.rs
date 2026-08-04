use std::ops::Deref;
use std::ops::DerefMut;

use crate::window::SdlWindow;
use crate::window::WindowId;

pub struct PlatformWindow {
    sdl: SdlWindow,
}

impl Deref for PlatformWindow {
    type Target = SdlWindow;

    fn deref(&self) -> &Self::Target {
        &self.sdl
    }
}

impl DerefMut for PlatformWindow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sdl
    }
}

impl PlatformWindow {
    pub fn new(sdl: SdlWindow) -> Self {
        Self { sdl }
    }
}
