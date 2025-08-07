use crate::{time::Time, window::Window};

pub struct Context {
    pub window: Window,
    pub time: Time,
    running: bool,
    scene_change_request: Option<String>,
}

impl Context {
    pub(crate) fn new(window: Window) -> Self {
        let time = Time::new();

        Self {
            window,
            time,
            running: true,
            scene_change_request: None,
        }
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    /// Request a scene change. This will be processed at the end of the frame.
    pub fn switch_scene<S: Into<String>>(&mut self, scene_name: S) {
        self.scene_change_request = Some(scene_name.into());
    }

    /// Used internally by the engine to get and clear pending scene change requests
    pub(crate) fn take_scene_change_request(&mut self) -> Option<String> {
        self.scene_change_request.take()
    }
}
