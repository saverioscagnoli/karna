use nostd::alloc::boxed::Box;
use nostd::alloc::string::String;
use nostd::alloc::string::ToString;
use sdl3::render::Color;
use sdl3::window::WindowId;

use crate::builder::WindowBuilder;
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
    resizable: bool,
    decorated: bool,
    always_on_top: bool,
    transparent: bool,
    opacity: f32,
    focusable: bool,
    high_pixel_density: bool,
    grab_mouse: bool,
    grab_keyboard: bool,
    clear_color: Color,
}

impl WindowData {
    pub(crate) fn init(sdl_window: &mut SdlWindow, b: &WindowBuilder) -> Self {
        sdl_window.set_resizable(b.resizable);
        sdl_window.set_decorated(b.decorated);
        sdl_window.set_always_on_top(b.always_on_top);
        sdl_window.set_opacity(b.opacity);
        sdl_window.set_focusable(b.focusable);
        sdl_window.set_mouse_grabbed(b.grab_mouse);
        sdl_window.set_keyboard_grabbed(b.grab_keyboard);

        Self {
            id: sdl_window.id(),
            title: sdl_window.title().to_string(),
            size: sdl_window.size(),
            pixel_size: sdl_window.pixel_size(),
            mouse_poistion: math::vec2!(0.0, 0.0),
            mouse_delta: math::vec2!(0.0, 0.0),
            resizable: sdl_window.is_resizable(),
            decorated: sdl_window.is_decorated(),
            always_on_top: sdl_window.is_always_on_top(),
            transparent: sdl_window.is_transparent(),
            opacity: sdl_window.opacity(),
            focusable: sdl_window.is_focusable(),
            high_pixel_density: sdl_window.is_high_pixel_density(),
            grab_mouse: sdl_window.mouse_grabbed(),
            grab_keyboard: sdl_window.keyboard_grabbed(),
            clear_color: Color::BLACK,
        }
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn update_input(&mut self, pos: math::Vector2<f32>, d: math::Vector2<f32>) {
        self.mouse_poistion = pos;
        self.mouse_delta += d;
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

    #[inline]
    pub fn clear_color(&self) -> Color {
        self.clear_color
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

    pub fn is_resizable(&self) -> bool {
        self.data.resizable
    }

    pub fn is_decorated(&self) -> bool {
        self.data.decorated
    }

    pub fn is_always_on_top(&self) -> bool {
        self.data.always_on_top
    }

    pub fn is_transparent(&self) -> bool {
        self.data.transparent
    }

    pub fn opacity(&self) -> f32 {
        self.data.opacity
    }

    pub fn is_focusable(&self) -> bool {
        self.data.focusable
    }

    pub fn is_high_pixel_density(&self) -> bool {
        self.data.high_pixel_density
    }

    pub fn mouse_grabbed(&self) -> bool {
        self.data.grab_mouse
    }

    pub fn keyboard_grabbed(&self) -> bool {
        self.data.grab_keyboard
    }

    pub fn clear_color(&self) -> Color {
        self.data.clear_color
    }

    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.data.clear_color = color.into();
    }
}
