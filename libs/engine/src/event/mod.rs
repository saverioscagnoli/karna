mod event;
mod outbox;

pub use event::*;
use nostd::alloc::vec::Vec;
pub use outbox::*;

pub struct AppOutboxes {
    pub window: Outbox<AppEvent>,
    pub time: Outbox<AppEvent>,
}

impl AppOutboxes {
    pub fn new() -> Self {
        Self {
            window: Outbox::new("window", 25),
            time: Outbox::new("time", 25),
        }
    }

    pub fn total_cap(&self) -> usize {
        self.window.cap + self.time.cap
    }

    pub fn drain_into(&mut self, out: &mut Vec<AppEvent>) {
        let Self { window, time } = self;

        window.drain_into(out);
        time.drain_into(out);
    }
}
