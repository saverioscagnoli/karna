use nostd::alloc::boxed::Box;
use nostd::alloc::string::String;
use nostd::alloc::string::ToString;
use sdl3::window::WindowId;

use crate::event::AppEvent;
use crate::event::Outbox;
use crate::event::WindowEvent;

pub type SdlWindow = sdl3::window::Window;

pub struct WindowData {
    id: WindowId,
    title: String,
    size: math::Size<u32>,
    pixel_size: math::Size<u32>,
    mouse_poistion: math::Vector2<f32>,
    mouse_delta: math::Vector2<f32>,
}

impl WindowData {
    pub(crate) fn init(sdl_window: &SdlWindow) -> Self {
        Self {
            id: sdl_window.id(),
            title: sdl_window.title().to_string(),
            size: sdl_window.size(),
            pixel_size: sdl_window.pixel_size(),
            mouse_poistion: math::vec2!(0.0, 0.0),
            mouse_delta: math::vec2!(0.0, 0.0),
        }
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn update_input(&mut self, pos: math::Vector2<f32>, d: math::Vector2<f32>) {
        self.mouse_poistion = pos;
        self.mouse_delta = d;
    }

    #[inline]
    fn roll_input(&mut self) {
        self.mouse_delta.set([0.0, 0.0]);
    }

    #[inline]
    pub fn sync(&mut self, window: &SdlWindow) {
        self.title = window.title().into();
        self.size = window.size();
        self.roll_input();
    }
}

pub struct Window<'a> {
    pub(crate) data: &'a mut WindowData,
    pub(crate) outbox: &'a mut Outbox<AppEvent>,
}

impl<'a> Window<'a> {
    pub fn id(&self) -> WindowId {
        self.data.id
    }

    pub fn title(&self) -> &str {
        &self.data.title
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: Into<Box<str>>,
    {
        let str = title.into();

        self.data.title = str.to_string();
        self.outbox.push(AppEvent::Window {
            window: self.data.id,
            wevent: WindowEvent::SetTitle(str),
        });
    }

    pub fn size(&self) -> math::Size<u32> {
        self.data.size
    }

    pub fn pixel_size(&self) -> math::Size<u32> {
        self.data.pixel_size
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size = size.into();

        self.data.size = size;
        self.outbox.push(AppEvent::Window {
            window: self.data.id,
            wevent: WindowEvent::SetSize(size),
        });
    }

    pub fn mouse_position(&self) -> math::Vector2<f32> {
        self.data.mouse_poistion
    }

    pub fn mouse_delta(&self) -> math::Vector2<f32> {
        self.data.mouse_delta
    }
}
