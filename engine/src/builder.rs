use std::mem;

use crate::App;
use crate::scene::Scene;

pub struct WindowBuilder {
    pub(crate) title: String,
    pub(crate) size: math::Size<u32>,
    pub(crate) scenes: Vec<(u32, Box<dyn Scene>)>,
    pub(crate) active_scenes: Vec<u32>,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: String::from("My Window"),
            size: math::Size::new(800, 600),
            scenes: Vec::new(),
            active_scenes: Vec::new(),
        }
    }
}

impl WindowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title = title.into();
        self
    }

    pub fn with_size<S>(mut self, size: S) -> Self
    where
        S: Into<math::Size<u32>>,
    {
        self.size = size.into();
        self
    }

    pub fn with_scene<S>(mut self, id: u32, scene: S, active: bool) -> Self
    where
        S: Scene + 'static,
    {
        self.scenes.push((id, Box::new(scene)));

        if active {
            self.active_scenes.push(id);
        }

        self
    }
}

#[derive(Default)]
pub struct AppBuilder {
    windows: Vec<WindowBuilder>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_window(mut self, b: WindowBuilder) -> Self {
        self.windows.push(b);
        self
    }

    pub fn build(mut self) -> App {
        let mut app = App::new();

        for b in mem::take(&mut self.windows) {
            app.queued_windows.push(b);
        }

        app
    }
}
