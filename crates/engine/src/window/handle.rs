use math as m;
use std::rc::Rc;

use crate::Color;
use crate::Cursor;
use crate::SceneId;
use crate::events::UserEvent;
use crate::events::WindowId;
use crate::events::queue::EventDispatcher;
use crate::events::user::UserWindowEvent;
use crate::gpu::PresentMode;
use crate::window::Window;

pub struct WindowHandle {
    pub(crate) id: WindowId,
    pub(crate) title: Rc<str>,
    pub(crate) size: m::Size<u32>,
    pub(crate) resizable: bool,
    pub(crate) mouse_position: m::Vector2<f32>,
    pub(crate) mouse_delta: m::Vector2<f32>,
    pub(crate) clear_color: Color,
    pub(crate) present_mode: PresentMode,
    pub(crate) dispatcher: EventDispatcher<UserEvent>,
}

impl WindowHandle {
    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn load_scene(&self, scene: SceneId) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::LoadScene(scene),
        });
    }

    pub fn unload_scene(&self, scene: SceneId) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::UnloadScene(scene),
        });
    }

    pub fn activate_scene(&self, scene: SceneId) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::ActivateScene(scene),
        });
    }

    pub fn deactivate_scene(&self, scene: SceneId) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::DeactivateScene(scene),
        });
    }

    pub fn start_text_input(&self) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::StartTextInput,
        });
    }

    pub fn stop_text_input(&self) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::StopTextInput,
        });
    }

    pub fn set_text_input_area<O, S>(&self, origin: O, size: S, cursor: i32)
    where
        O: Into<m::Vector2<i32>>,
        S: Into<m::Size<u32>>,
    {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::SetTextInputArea {
                origin: origin.into(),
                size: size.into(),
                cursor,
            },
        });
    }

    pub fn clear_text_input_area(&self) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::ClearTextInputArea,
        });
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

    pub fn set_cursor(&self, cursor: Cursor) {
        self.dispatcher.dispatch(UserEvent::ChangeCursor(cursor));
    }

    pub fn set_present_mode(&mut self, mode: PresentMode) {
        self.dispatcher.dispatch(UserEvent::Window {
            id: self.id,
            wevent: UserWindowEvent::SetPresentMode(mode),
        });
    }

    pub(crate) fn sync(&mut self, window: &Window) {
        self.title = window.title().into();
        self.size = window.size();
    }

    pub(crate) fn roll_mouse(&mut self) {
        self.mouse_delta.set([0.0, 0.0]);
    }
}
