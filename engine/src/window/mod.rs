pub mod audio;
pub mod clock;
pub mod context;
pub mod input;
pub mod pacer;
pub mod platform;
pub mod resources;
pub mod state;
pub mod time;

use std::rc::Rc;

use utils::WindowId;

use crate::event::AppEvent;
use crate::event::EventDispatcher;
use crate::window::platform::PlatformWindow;

#[derive(Debug)]
pub enum WindowCommand {
    SetWindowSize(math::Size<u32>),
    SetWindowTitle(Rc<str>),
    SetResizable(bool),
}

#[derive(Debug)]
pub struct WindowCommandRequest {
    pub window: WindowId,
    pub command: WindowCommand,
}

pub struct WindowHandle {
    id: WindowId,
    size: math::Size<u32>,
    pixel_size: math::Size<u32>,
    title: Rc<str>,
    resizable: bool,
    events: EventDispatcher<AppEvent>,
}

impl WindowHandle {
    fn push(&self, command: WindowCommand) {
        self.events.send(AppEvent::Window(WindowCommandRequest {
            window: self.id,
            command,
        }));
    }

    pub(crate) fn new(
        id: WindowId,
        size: math::Size<u32>,
        pixel_size: math::Size<u32>,
        title: &str,
        resizable: bool,
        events: EventDispatcher<AppEvent>,
    ) -> Self {
        Self {
            id,
            size,
            pixel_size,
            title: title.into(),
            resizable,
            events,
        }
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size: math::Size<u32> = size.into();

        self.size = size;
        self.push(WindowCommand::SetWindowSize(size));
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        let title: Rc<str> = title.as_ref().into();

        self.title = Rc::clone(&title);
        self.push(WindowCommand::SetWindowTitle(title));
    }

    pub fn is_resizable(&self) -> bool {
        self.resizable
    }

    pub fn set_resizable(&mut self, v: bool) {
        self.resizable = v;
        self.push(WindowCommand::SetResizable(v));
    }

    pub(crate) fn sync(&mut self, platform: &PlatformWindow) {
        self.size = platform.size();
        self.pixel_size = platform.pixel_size();
        self.resizable = platform.is_resizable();

        if &self.title != platform.title() {
            self.title = Rc::clone(platform.title());
        }
    }
}
