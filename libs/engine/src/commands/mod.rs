mod command;
mod outbox;

pub use command::*;
pub use outbox::*;
use sdl3::window::WindowId;

pub struct AppOutboxes {
    pub window: Outbox<(WindowId, WindowCommand)>,
    pub time: Outbox<TimeCommand>,
    pub scene: Outbox<(WindowId, SceneCommand)>,
    pub input: Outbox<InputCommand>,
    pub audio: Outbox<AudioCommand>,
}

impl AppOutboxes {
    pub fn new() -> Self {
        Self {
            window: Outbox::new("window", 32),
            time: Outbox::new("time", 32),
            scene: Outbox::new("scene", 32),
            input: Outbox::new("input", 32),
            audio: Outbox::new("audio", 32),
        }
    }

    pub fn total_cap(&self) -> usize {
        self.window.cap() + self.time.cap() + self.scene.cap() + self.input.cap()
    }
}
