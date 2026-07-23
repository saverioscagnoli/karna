use std::{
    any::Any,
    sync::mpsc::{Sender, channel},
    thread::{self, JoinHandle},
};

use utils::{FastHashMap, WindowId};

use crate::window::{SdlEvent, SdlWindow, WindowAction};

mod window;

pub struct WindowThread {
    window: SdlWindow,
    shutdown_signal: Sender<()>,
    handle: JoinHandle<()>,
}

pub struct App {
    threads: FastHashMap<WindowId, WindowThread>,
    sdl: sdl3::Sdl,
    video: sdl3::VideoSubsystem,
    events: sdl3::EventSubsystem,
}

impl App {
    fn new() -> Self {
        let sdl = sdl3::init().expect("Failed to init sdl3");
        let video = sdl.video().expect("Failed to init video subsystem");
        let events = sdl.event().expect("Failed to init event subsystem");

        Self {
            threads: FastHashMap::default(),
            sdl,
            video,
            events,
        }
    }

    fn spawn_window(&mut self, title: &str, size: math::Size<u32>) {
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

        self.threads.insert(
            window_id,
            WindowThread {
                window,
                shutdown_signal: shutdown_tx,
                handle,
            },
        );
    }

    fn run(mut self) {
        self.spawn_window("xd", (800, 600).into());

        let mut pump = self.sdl.event_pump().expect("Failed to get event pump");
    }
}
