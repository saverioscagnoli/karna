pub mod context;
pub mod pacer;
pub mod platform;
pub mod state;
pub mod time;

use std::rc::Rc;

use crate::Color;
use crate::conf::config;
use crate::event::AppEvent;
use crate::event::EventDispatcher;
use crate::event::WindowEvent;
use crate::window::platform::PlatformWindow;
use crate::window::state::WindowState;

pub type SdlWindow = sdl3::video::Window;
pub type WindowId = u32;

pub struct WindowHandle {
    pub(crate) id: WindowId,
    pub(crate) title: Rc<str>,
    pub(crate) size: math::Size<u32>,
    pub(crate) pixel_size: math::Size<u32>,
    pub(crate) clear_color: Color,
    pub(crate) mouse_position: math::Vector2<f32>,
    pub(crate) mouse_delta: math::Vector2<f32>,
    pub(crate) dispatcher: EventDispatcher<AppEvent>,
}

impl WindowHandle {
    pub(crate) fn new(window: &PlatformWindow, dispatcher: EventDispatcher<AppEvent>) -> Self {
        let conf = config();

        Self {
            id: window.id(),
            title: window.title().into(),
            size: window.size().into(),
            pixel_size: window.size_in_pixels().into(),
            clear_color: conf.clear_color,
            mouse_position: math::Vector2::zero(),
            mouse_delta: math::Vector2::zero(),
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

    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    pub fn clear_color_mut(&mut self) -> &mut Color {
        &mut self.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.clear_color = color.into();
    }

    pub fn mouse_position(&self) -> math::Vector2<f32> {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> math::Vector2<f32> {
        self.mouse_delta
    }

    pub(crate) fn roll_frame(&mut self) {
        self.mouse_delta.set([0.0, 0.0]);
    }
}

pub struct WindowEntry {
    pub window: PlatformWindow,
    pub state: WindowState,
}
