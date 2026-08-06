use std::ops::Deref;
use std::ops::DerefMut;

use logging::warn;

use crate::event::WindowEvent;
use crate::window::SdlWindow;
use crate::window::pacer::FramePacer;

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

    pub fn handle_event(&mut self, event: WindowEvent, pacer: &mut FramePacer) {
        match event {
            WindowEvent::TitleChangeRequested(t) => {
                if let Err(e) = self.set_title(&t) {
                    warn!("Failed to change window title: {}", e);
                }
            }

            WindowEvent::SizeChangeRequested(s) => {
                if let Err(e) = self.set_size(s.width, s.height) {
                    warn!("Failed to change window size: {}", e);
                }
            }

            WindowEvent::FpsTargetChangeRequested(t) => {
                pacer.set_target_fps(t);
            }
        };
    }
}
