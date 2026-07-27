use crate::window::time::TimeCommand;

pub type SdlEvent = sdl3::event::Event;
pub type SdlWindowEvent = sdl3::event::WindowEvent;

pub enum AppEvent {
    Time(TimeCommand),
}
