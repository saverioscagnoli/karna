use nostd::alloc::boxed::Box;
use nostd::alloc::string::String;
use nostd::alloc::vec::Vec;
use nostd::collections::HashMap;
use traccia::debug;

use crate::App;
use crate::scene::Scene;
use crate::scene::SceneBuilder;
use crate::scene::SceneId;

pub struct WindowBuilder {
    pub title: String,
    pub size: math::Size<u32>,
    pub resizable: bool,
    pub scene_builders: HashMap<SceneId, SceneBuilder>,
    pub active_scenes: Vec<SceneId>,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: String::from("My Window"),
            size: math::size!(800, 600),
            resizable: false,
            scene_builders: HashMap::default(),
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

    pub fn with_scene<S>(mut self, id: SceneId) -> Self
    where
        S: Scene,
    {
        self.scene_builders
            .insert(id, Box::new(|ctx| Box::new(S::load(ctx))));
        self
    }

    pub fn with_active_scene(mut self, id: SceneId) -> Self {
        self.active_scenes.push(id);
        self
    }
}

pub struct AppBuilder {
    windows: Vec<WindowBuilder>,
    root: String,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            root: String::from("."),
        }
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_window(mut self, builder: WindowBuilder) -> Self {
        self.windows.push(builder);
        self
    }

    pub fn with_root<P>(mut self, root: P) -> Self
    where
        P: Into<String>,
    {
        self.root = root.into();
        self
    }

    pub fn build(self) -> App {
        debug!("Requested creation of {} window(s)", self.windows.len());
        debug!("Resolved root path: {}", self.root);

        let mut app = App::new(self.root);

        for builder in self.windows {
            app.requested_windows.push(builder)
        }

        app
    }
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }
}
