use core::ops::Deref;

use nostd::alloc::boxed::Box;
use nostd::alloc::string::String;
use nostd::alloc::string::ToString;
use sdl3::render::Color;
use sdl3::window::FullscreenMode;
use sdl3::window::WindowFlags;
use sdl3::window::WindowId;
use sdl3::window::WindowState;

use crate::event::AppEvent;
use crate::event::Outbox;
use crate::event::WindowEvent;

pub type SdlWindow = sdl3::window::Window;

pub struct WindowData {
    id: WindowId,
    title: String,
    size: math::Size<u32>,
    pixel_size: math::Size<u32>,
    opacity: f32,
    mouse_poistion: math::Vector2<f32>,
    mouse_delta: math::Vector2<f32>,
    flags: WindowFlags,
    clear_color: Color,
}

impl Deref for WindowData {
    type Target = WindowFlags;

    fn deref(&self) -> &Self::Target {
        &self.flags
    }
}

impl WindowData {
    pub(crate) fn init(window: &SdlWindow) -> Self {
        Self {
            id: window.id(),
            title: window.title().to_string(),
            size: window.size(),
            pixel_size: window.pixel_size(),
            opacity: window.opacity(),
            flags: window.flags(),
            mouse_poistion: math::vec2!(0.0, 0.0),
            mouse_delta: math::vec2!(0.0, 0.0),
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
        self.opacity = window.opacity();
        self.flags = window.flags();
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
    #[inline]
    fn push(&mut self, wevent: WindowEvent) {
        self.outbox.push(AppEvent::Window {
            window: self.data.id,
            wevent,
        });
    }

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
        self.push(WindowEvent::SetSize(size));
    }

    pub fn mouse_position(&self) -> math::Vector2<f32> {
        self.data.mouse_poistion
    }

    pub fn mouse_delta(&self) -> math::Vector2<f32> {
        self.data.mouse_delta
    }

    pub fn is_windowed(&self) -> bool {
        self.data.state == WindowState::Normal
    }

    pub fn set_windowed(&mut self) {
        self.push(WindowEvent::SetWindowState(WindowState::Normal));
    }

    pub fn is_maximized(&self) -> bool {
        self.data.state == WindowState::Maximized
    }

    pub fn set_maximized(&mut self) {
        self.push(WindowEvent::SetWindowState(WindowState::Maximized));
    }

    pub fn is_minimized(&self) -> bool {
        self.data.state == WindowState::Minimized
    }

    pub fn set_minimized(&mut self) {
        self.push(WindowEvent::SetWindowState(WindowState::Minimized));
    }

    pub fn is_fullscreen(&self) -> bool {
        self.data.state == WindowState::Fullscreen
    }

    pub fn set_fullscreen(&mut self, mode: FullscreenMode) {
        self.push(WindowEvent::SetFullscreen(mode));
    }

    pub fn restore(&mut self) {
        self.push(WindowEvent::Restore)
    }

    pub fn is_hidden(&self) -> bool {
        self.data.hidden
    }

    pub fn set_hidden(&mut self, hidden: bool) {
        self.push(WindowEvent::SetHidden(hidden));
    }

    pub fn is_resizable(&self) -> bool {
        self.data.resizable
    }

    pub fn set_resizable(&mut self, resizable: bool) {
        self.push(WindowEvent::SetResizable(resizable));
    }

    pub fn is_decorated(&self) -> bool {
        self.data.decorated
    }

    pub fn set_decorated(&mut self, decorated: bool) {
        self.push(WindowEvent::SetDecorated(decorated));
    }

    pub fn is_always_on_top(&self) -> bool {
        self.data.always_on_top
    }

    pub fn set_always_on_top(&mut self, on_top: bool) {
        self.push(WindowEvent::SetAlwaysOnTop(on_top));
    }

    pub fn is_utility(&self) -> bool {
        self.data.utility
    }

    pub fn is_transparent(&self) -> bool {
        self.data.transparent
    }

    pub fn opacity(&self) -> f32 {
        self.data.opacity
    }

    pub fn set_opacity(&mut self, value: f32) {
        self.push(WindowEvent::SetOpacity(value));
    }

    pub fn is_focusable(&self) -> bool {
        self.data.focusable
    }

    pub fn set_focusable(&mut self, focusable: bool) {
        self.push(WindowEvent::SetFocusable(focusable))
    }

    pub fn is_high_pixel_density(&self) -> bool {
        self.data.high_pixel_density
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        self.data.mouse_grabbed
    }

    pub fn set_mouse_grabbed(&mut self, grab: bool) {
        self.push(WindowEvent::SetMouseGrabbed(grab))
    }

    pub fn is_keyboard_grabbed(&self) -> bool {
        self.data.keyboard_grabbed
    }

    pub fn set_keyboard_grabbed(&mut self, grab: bool) {
        self.push(WindowEvent::SetKeyboardGrabbed(grab));
    }

    pub fn is_relative_mouse(&self) -> bool {
        self.data.relative_mouse
    }

    pub fn set_relative_mouse(&mut self, relative: bool) {
        self.push(WindowEvent::SetRelativeMouse(relative))
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
