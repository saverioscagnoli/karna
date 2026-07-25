use logging::debug;
use utils::FastHashMap;

use crate::{App, scene::Scene};

pub struct WindowBuilder {
    pub title: String,
    pub size: math::Size<u32>,
    pub scenes: FastHashMap<String, Box<dyn Scene>>,
    pub active_scenes: Vec<String>,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: String::from("My Window"),
            size: math::Size::new(800, 600),
            scenes: FastHashMap::default(),
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

    pub fn with_scene<T, S>(mut self, label: T, scene: S, active: bool) -> Self
    where
        T: Into<String>,
        S: Scene + 'static,
    {
        let label: String = label.into();
        self.scenes.insert(label.clone(), Box::new(scene));

        if active {
            self.active_scenes.push(label)
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

    pub fn with_window(mut self, builder: WindowBuilder) -> Self {
        self.windows.push(builder);
        self
    }

    pub fn build(self) -> App {
        let mut app = App::new();
        let len = self.windows.len();

        debug!("Requested creation of {} window(s).", len);

        for b in self.windows {
            app.queued_windows.push(b);
        }

        app
    }
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }
}
