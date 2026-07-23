mod render;
mod simulation;

use std::sync::mpsc::channel;
use std::thread::{self};

use crate::window::SdlEvent;
use crate::window::WindowAction;

mod window;

pub struct App {
    sdl: sdl3::Sdl,
    video: sdl3::VideoSubsystem,
    events: sdl3::EventSubsystem,
}

impl App {
    fn new() -> Self {
        let sdl = sdl3::init().expect("Failed to init sdl3");
        let video = sdl.video().expect("Failed to init video subsystem");
        let events = sdl.event().expect("Failed to init event subsystem");

        Self { sdl, video, events }
    }

    fn spawn_windows(&mut self, title: &str, size: math::Size<u32>) {
        let window = self
            .video
            .window(title, size.width, size.height)
            .build()
            .expect("Failed to create window");

        let window_id = window.id();

        // Channels
        let (event_tx, event_rx) = channel::<SdlEvent>();
        let (action_tx, action_rx) = channel::<WindowAction>();
        let (shutdown_tx, shutdown_rx) = channel::<()>();

        let handle = thread::spawn(move || {});
    }

    fn run(mut self) {
        let mut pump = self.sdl.event_pump().expect("Failed to get event pump");
    }
}
