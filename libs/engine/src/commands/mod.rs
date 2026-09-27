mod command;
mod outbox;

pub use command::*;
pub use outbox::*;
use sdl3::window::WindowId;

pub struct AppOutboxes {
    pub window: Outbox<(WindowId, WindowCommand)>,
    pub time: Outbox<TimeCommand>,
    pub scene: Outbox<(WindowId, SceneCommand)>,
}

impl AppOutboxes {
    pub fn new() -> Self {
        Self {
            window: Outbox::new("window", 25),
            time: Outbox::new("time", 25),
            scene: Outbox::new("scene", 25),
        }
    }

    pub fn total_cap(&self) -> usize {
        self.window.cap() + self.time.cap() + self.scene.cap()
    }
}
