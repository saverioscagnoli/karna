use std::path::PathBuf;

use logging::debug;
use math as m;
use utils::FastHashMap;

use crate::App;
use crate::config::config;
use crate::path::resolve_base_path;
use crate::scene::Scene;
use crate::scene::SceneBuilder;
use crate::scene::SceneId;

pub struct WindowBuilder {
    pub title: String,
    pub size: m::Size<u32>,
    pub resizable: bool,
    pub scene_builders: FastHashMap<SceneId, SceneBuilder>,
    pub scenes_active: Vec<SceneId>,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        let config = config();

        Self {
            title: config.window.title.clone(),
            size: config.window.size,
            resizable: config.window.resizable,
            scene_builders: FastHashMap::default(),
            scenes_active: Vec::new(),
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
        S: Into<m::Size<u32>>,
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
        self.scenes_active.push(id);
        self
    }
}

pub struct AppBuilder {
    windows: Vec<WindowBuilder>,
    root: PathBuf,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            root: resolve_base_path(),
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
        P: Into<PathBuf>,
    {
        self.root = root.into();
        self
    }

    pub fn build(self) -> App {
        debug!("Requested creation of {} window(s)", self.windows.len());
        debug!("Resolved root path: {}", self.root.display());

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
