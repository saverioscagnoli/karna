pub mod context;
pub mod platform;
pub mod state;

use std::rc::Rc;

use crate::event::AppEvent;
use crate::event::EventDispatcher;
use crate::event::WindowEvent;
use crate::window::platform::PlatformWindow;
use crate::window::state::WindowState;

pub type SdlWindow = sdl3::video::Window;
pub type WindowId = u32;

pub struct WindowHandle {
    id: WindowId,
    title: Rc<str>,
    size: math::Size<u32>,
    pixel_size: math::Size<u32>,
    resizable: bool,
    dispatcher: EventDispatcher<AppEvent>,
}

impl WindowHandle {
    pub(crate) fn new(window: &PlatformWindow, dispatcher: EventDispatcher<AppEvent>) -> Self {
        Self {
            id: window.id(),
            title: window.title().into(),
            size: window.size().into(),
            pixel_size: window.size_in_pixels().into(),
            resizable: false,
            dispatcher,
        }
    }

    fn send(&self, event: WindowEvent) {
        self.dispatcher
            .send(AppEvent::Window { id: self.id, event });
    }

    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        let title: Rc<str> = title.as_ref().into();

        self.send(WindowEvent::TitleChangeRequested(title.clone()));
        self.title = title;
    }

    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size: math::Size<u32> = size.into();

        self.send(WindowEvent::SizeChangeRequested(size));
        self.size = size;
    }

    pub fn pixel_size(&self) -> math::Size<u32> {
        self.pixel_size
    }

    pub fn pixel_width(&self) -> u32 {
        self.pixel_size.width
    }

    pub fn pixel_height(&self) -> u32 {
        self.pixel_size.height
    }
}

pub struct WindowEntry {
    pub window: PlatformWindow,
    pub state: WindowState,
}
