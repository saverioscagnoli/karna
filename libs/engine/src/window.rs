use core::ops::Deref;

use nostd::alloc::boxed::Box;
use nostd::alloc::string::String;
use nostd::alloc::string::ToString;
use sdl3::gpu::PresentMode;
use sdl3::monitor::Monitor as SdlMonitor;
use sdl3::render::Color;
use sdl3::window::FullscreenMode;
use sdl3::window::WindowFlags;
use sdl3::window::WindowId;
use sdl3::window::WindowState;

use crate::commands::Outbox;
use crate::commands::WindowCommand;
use crate::monitors::Monitor;

pub type SdlWindow = sdl3::window::Window;

pub struct WindowData {
    id: WindowId,
    title: String,
    size: math::Size<u32>,
    pixel_size: math::Size<u32>,
    mouse_poistion: math::Vector2<f32>,
    mouse_delta: math::Vector2<f32>,
    opacity: f32,
    present_mode: PresentMode,
    flags: WindowFlags,
    clear_color: Color,
    monitor: Option<Monitor>,
}

impl WindowData {
    pub(crate) fn init(window: &SdlWindow) -> Self {
        Self {
            id: window.id(),
            title: window.title().to_string(),
            size: window.size(),
            pixel_size: window.pixel_size(),
            mouse_poistion: math::vec2!(0.0, 0.0),
            mouse_delta: math::vec2!(0.0, 0.0),
            opacity: window.opacity(),
            present_mode: window.present_mode(),
            flags: window.flags(),
            clear_color: Color::BLACK,
            monitor: SdlMonitor::for_window(window).map(Monitor::from),
        }
    }

    #[inline]
    pub(crate) fn update_input(&mut self, pos: math::Vector2<f32>, d: math::Vector2<f32>) {
        self.mouse_poistion = pos;
        self.mouse_delta += d;
    }

    #[inline]
    fn roll_input(&mut self) {
        self.mouse_delta.set([0.0, 0.0]);
    }

    #[inline]
    pub(crate) fn sync(&mut self, window: &SdlWindow) {
        self.title = window.title().into();
        self.size = window.size();
        self.pixel_size = window.pixel_size();
        self.opacity = window.opacity();
        self.flags = window.flags();
        self.monitor = SdlMonitor::for_window(window).map(Monitor::from);
        self.roll_input();
    }

    #[inline]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[inline]
    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    #[inline]
    pub fn pixel_size(&self) -> math::Size<u32> {
        self.pixel_size
    }

    #[inline]
    pub fn mouse_position(&self) -> math::Vector2<f32> {
        self.mouse_poistion
    }

    #[inline]
    pub fn mouse_delta(&self) -> math::Vector2<f32> {
        self.mouse_delta
    }

    #[inline]
    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }

    #[inline]
    pub fn is_windowed(&self) -> bool {
        self.flags.state == WindowState::Normal
    }

    #[inline]
    pub fn is_maximized(&self) -> bool {
        self.flags.state == WindowState::Maximized
    }

    #[inline]
    pub fn is_minimized(&self) -> bool {
        self.flags.state == WindowState::Minimized
    }

    #[inline]
    pub fn is_fullscreen(&self) -> bool {
        self.flags.state == WindowState::Fullscreen
    }

    #[inline]
    pub fn is_hidden(&self) -> bool {
        self.flags.hidden
    }

    #[inline]
    pub fn is_resizable(&self) -> bool {
        self.flags.resizable
    }

    #[inline]
    pub fn is_decorated(&self) -> bool {
        self.flags.decorated
    }

    #[inline]
    pub fn is_always_on_top(&self) -> bool {
        self.flags.always_on_top
    }

    #[inline]
    pub fn is_utility(&self) -> bool {
        self.flags.utility
    }

    #[inline]
    pub fn is_transparent(&self) -> bool {
        self.flags.transparent
    }

    #[inline]
    pub fn is_focusable(&self) -> bool {
        self.flags.focusable
    }

    #[inline]
    pub fn is_high_pixel_density(&self) -> bool {
        self.flags.high_pixel_density
    }

    #[inline]
    pub fn is_mouse_grabbed(&self) -> bool {
        self.flags.mouse_grabbed
    }

    #[inline]
    pub fn is_keyboard_grabbed(&self) -> bool {
        self.flags.keyboard_grabbed
    }

    #[inline]
    pub fn is_relatve_mouse(&self) -> bool {
        self.flags.relative_mouse
    }

    #[inline]
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    #[inline]
    pub fn clear_color(&self) -> Color {
        self.clear_color
    }

    #[inline]
    pub fn monitor(&self) -> Option<Monitor> {
        self.monitor
    }
}

pub struct Window<'a> {
    pub(crate) data: &'a mut WindowData,
    pub(crate) outbox: &'a mut Outbox<(WindowId, WindowCommand)>,
}

impl Deref for Window<'_> {
    type Target = WindowData;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a> Window<'a> {
    #[inline]
    fn push(&mut self, command: WindowCommand) {
        self.outbox.push((self.id, command));
    }

    pub fn set_title<T>(&mut self, title: T)
    where
        T: Into<Box<str>>,
    {
        let str = title.into();

        self.data.title = str.to_string();
        self.push(WindowCommand::SetTitle(str));
    }

    pub fn set_size<S>(&mut self, size: S)
    where
        S: Into<math::Size<u32>>,
    {
        let size = size.into();

        self.data.size = size;
        self.push(WindowCommand::SetSize(size));
    }

    #[inline]
    pub fn set_opacity(&mut self, value: f32) {
        self.push(WindowCommand::SetOpacity(value));
    }

    #[inline]
    pub fn set_present_mode(&mut self, mode: PresentMode) {
        self.push(WindowCommand::SetPresentMode(mode));
    }

    #[inline]
    pub fn set_clear_color<C>(&mut self, color: C)
    where
        C: Into<Color>,
    {
        self.data.clear_color = color.into();
    }

    #[inline]
    pub fn set_windowed(&mut self) {
        self.push(WindowCommand::SetState(WindowState::Normal));
    }

    #[inline]
    pub fn set_maximized(&mut self) {
        self.push(WindowCommand::SetState(WindowState::Maximized));
    }

    #[inline]
    pub fn set_minimized(&mut self) {
        self.push(WindowCommand::SetState(WindowState::Minimized));
    }

    #[inline]
    pub fn set_fullscreen(&mut self, mode: FullscreenMode) {
        self.push(WindowCommand::SetFullscreenMode(mode));
    }

    #[inline]
    pub fn set_hidden(&mut self, hidden: bool) {
        self.push(WindowCommand::SetHidden(hidden));
    }

    #[inline]
    pub fn set_resizable(&mut self, resizable: bool) {
        self.push(WindowCommand::SetResizable(resizable));
    }

    #[inline]
    pub fn set_decorated(&mut self, decorated: bool) {
        self.push(WindowCommand::SetDecorated(decorated));
    }

    #[inline]
    pub fn set_always_on_top(&mut self, on_top: bool) {
        self.push(WindowCommand::SetAlwaysOnTop(on_top));
    }

    #[inline]
    pub fn set_focusable(&mut self, focusable: bool) {
        self.push(WindowCommand::SetFocusable(focusable))
    }

    #[inline]
    pub fn set_mouse_grabbed(&mut self, grab: bool) {
        self.push(WindowCommand::SetMouseGrabbed(grab))
    }

    #[inline]
    pub fn set_keyboard_grabbed(&mut self, grab: bool) {
        self.push(WindowCommand::SetKeyboardGrabbed(grab));
    }

    #[inline]
    pub fn set_relative_mouse(&mut self, relative: bool) {
        self.push(WindowCommand::SetRelativeMouse(relative))
    }

    #[inline]
    pub fn restore(&mut self) {
        self.push(WindowCommand::Restore)
    }
}
