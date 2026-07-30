use std::rc::Rc;

use logging::debug;
use logging::warn;
use sdl3::sys::video::SDL_SetWindowResizable;
use sdl3::sys::video::SDL_WINDOW_RESIZABLE;
use sdl3::video::Window;
use utils::WindowId;

use crate::window::WindowCommand;

pub struct PlatformWindow {
    sdl: Window,
    size: math::Size<u32>,
    pixel_size: math::Size<u32>,
    title: Rc<str>,
    resizable: bool,
    pending_resize: u8,
}

impl PlatformWindow {
    const RESIZE_GRACE_FRAMES: u8 = 8;

    pub fn new(sdl: Window) -> Self {
        let mut this = Self {
            sdl,
            size: math::Size::zero(),
            pixel_size: math::Size::zero(),
            title: "".into(),
            resizable: false,
            pending_resize: 0,
        };

        this.refresh();
        this
    }

    pub fn inner(&self) -> &Window {
        &self.sdl
    }

    pub fn id(&self) -> WindowId {
        self.sdl.id()
    }

    /// Logical size. Input coordinates are in these units.
    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    /// Framebuffer size. The camera projection wants this one.
    pub fn pixel_size(&self) -> math::Size<u32> {
        self.pixel_size
    }

    pub fn title(&self) -> &Rc<str> {
        &self.title
    }

    pub fn is_resizable(&self) -> bool {
        self.resizable
    }

    pub fn refresh(&mut self) {
        if self.pending_resize > 0 {
            self.pending_resize -= 1;
        } else {
            self.size = self.sdl.size().into();
        }

        self.pixel_size = self.sdl.size_in_pixels().into();

        let title = self.sdl.title();

        if &*self.title != title {
            self.title = title.into();
        }

        self.resizable = self.sdl.window_flags() & SDL_WINDOW_RESIZABLE == 0;
    }

    pub fn on_resized(&mut self, width: u32, height: u32) {
        self.size = math::Size::new(width, height);
        self.pending_resize = 0;
    }

    pub fn on_pixel_resized(&mut self, width: u32, height: u32) {
        self.pixel_size = math::Size::new(width, height);
    }

    pub fn apply(&mut self, command: WindowCommand) {
        match command {
            WindowCommand::SetWindowSize(size) => {
                if let Err(e) = self.sdl.set_size(size.width, size.height) {
                    warn!("Failed to set window size: {}", e);
                    return;
                }

                // Optimistic write
                self.size = size;
                self.pending_resize = Self::RESIZE_GRACE_FRAMES;

                debug!("Requested window size {:?}", size);
            }

            WindowCommand::SetWindowTitle(t) => {
                if let Err(e) = self.sdl.set_title(&t) {
                    warn!("Failed to set window title: {}", e);
                    return;
                }

                self.title = t;
                debug!("Set window title");
            }

            WindowCommand::SetResizable(v) => {
                unsafe { SDL_SetWindowResizable(self.sdl.raw(), v) };

                self.resizable = self.sdl.window_flags() & SDL_WINDOW_RESIZABLE != 0;

                if self.resizable != v {
                    warn!("Window manager declined resizable = {}", v);
                }

                debug!("Set window resizable to {}", self.resizable);
            }
        }
    }
}
