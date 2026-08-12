pub mod context;
pub mod pacer;
pub mod platform;
pub mod state;
pub mod time;

use std::rc::Rc;

use sdl3::SDL_Event;
use sdl3::SDL_EventType;
use sdl3::SDL_WindowID;

use crate::config::config;
use crate::events::AppEvent;
use crate::events::EventDispatcher;
use crate::events::WindowEvent;
use crate::render::color::Color;
use crate::window::platform::PlatformWindow;

pub type WindowId = SDL_WindowID;

pub struct WindowHandle {
    pub(crate) id: WindowId,
    pub(crate) title: Rc<str>,
    pub(crate) size: math::Size<u32>,
    pub(crate) mouse_position: math::Vector2<f32>,
    pub(crate) mouse_delta: math::Vector2<f32>,
    pub(crate) clear_color: Color,
    pub(crate) dispatcher: EventDispatcher<AppEvent>,
}

impl WindowHandle {
    pub fn new(window: &PlatformWindow, dispatcher: EventDispatcher<AppEvent>) -> Self {
        let config = config();

        Self {
            id: window.id(),
            title: window.title().into(),
            size: window.size(),
            mouse_position: math::Vector2::zero(),
            mouse_delta: math::Vector2::zero(),
            clear_color: config.window_clear_color,
            dispatcher,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: AsRef<str>,
    {
        let title: Rc<str> = title.as_ref().into();

        self.title = title.clone();
        self.dispatcher.send(AppEvent::Window {
            id: self.id,
            event: WindowEvent::TitleChangeRequested(title),
        });
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
        self.dispatcher.send(AppEvent::Window {
            id: self.id,
            event: WindowEvent::SizeChangeRequested(size),
        });
    }

    pub fn mouse_position(&self) -> math::Vector2<f32> {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> math::Vector2<f32> {
        self.mouse_delta
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

    pub(crate) fn handle_event(&mut self, event: &SDL_Event) {
        match SDL_EventType(unsafe { event.type_ }) {
            SDL_EventType::SDL_EVENT_MOUSE_MOTION => {
                let motion = unsafe { event.motion };

                self.mouse_position.set([motion.x, motion.y]);
                self.mouse_delta.set([
                    self.mouse_delta.x + motion.xrel,
                    self.mouse_delta.y + motion.yrel,
                ]);
            }

            _ => {}
        }
    }

    pub(crate) fn sync(&mut self, window: &PlatformWindow) {
        self.title = window.title().into();
        self.size = window.size();
    }

    /// Sets mouse delta to 0.
    /// This must be called at the end of each frame.
    pub(crate) fn roll_mouse(&mut self) {
        self.mouse_delta.set([0.0, 0.0]);
    }
}
