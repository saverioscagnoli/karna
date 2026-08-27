use math as m;
use std::rc::Rc;

use crate::events::UserEvent;
use crate::events::WindowId;
use crate::events::queue::EventDispatcher;
use crate::events::user::UserWindowEvent;
use crate::window::Window;

pub struct WindowHandle {
    pub id: WindowId,
    pub title: Rc<str>,
    pub size: m::Size<u32>,
    pub resizable: bool,
    pub mouse_position: m::Vector2<f32>,
    pub mouse_delta: m::Vector2<f32>,
    pub dispatcher: EventDispatcher<UserEvent>,
}

impl WindowHandle {
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

        self.title = title.clone();
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::ChangeTitle(title),
        });
    }

    pub fn size(&self) -> m::Size<u32> {
        self.size
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<m::Size<u32>>,
    {
        let size: m::Size<u32> = size.into();

        self.size = size;
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::ChangeSize(size),
        });
    }

    pub fn is_resizable(&self) -> bool {
        self.resizable
    }

    pub fn set_resizable(&mut self, value: bool) {
        self.resizable = value;
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::ChangeResizable(value),
        });
    }

    pub fn toggle_resizable(&mut self) {
        self.set_resizable(!self.resizable);
    }

    pub fn mouse_position(&self) -> m::Vector2<f32> {
        self.mouse_position
    }

    pub fn mouse_delta(&self) -> m::Vector2<f32> {
        self.mouse_delta
    }

    pub(crate) fn sync(&mut self, window: &Window) {
        self.title = window.title().into();
        self.size = window.size();
    }

    pub(crate) fn roll_mouse(&mut self) {
        self.mouse_delta.set([0.0, 0.0]);
    }
}
