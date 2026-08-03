use std::mem;
use std::path::PathBuf;

use logging::fatal;

use crate::App;
use crate::scene::{Scene, SceneFactory};

pub struct WindowBuilder {
    pub(crate) title: String,
    pub(crate) size: math::Size<u32>,
    pub(crate) resizable: bool,
    pub(crate) scenes: Vec<(u32, SceneFactory)>,
    pub(crate) active_scenes: Vec<u32>,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: String::from("My Window"),
            size: math::Size::new(800, 600),
            resizable: false,
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

    pub fn with_scene<S: Scene>(mut self, id: u32) -> Self {
        let id = id.into();

        if self.scenes.iter().any(|(existing, _)| *existing == id) {
            fatal!("scene id {id:?} registered twice");
        }

        self.scenes
            .push((id, Box::new(|ctx, s| Box::new(S::load(ctx, s)))));
        self
    }

    pub fn with_active_scene(mut self, id: u32) -> Self {
        let id = id.into();

        if !self.active_scenes.contains(&id) {
            self.active_scenes.push(id);
        }

        self
    }
}

pub struct AppBuilder {
    windows: Vec<WindowBuilder>,
    asset_workers: usize,
    asset_root: Option<PathBuf>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            asset_workers: 4,
            asset_root: None,
        }
    }
}

impl AppBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_window(mut self, b: WindowBuilder) -> Self {
        self.windows.push(b);
        self
    }

    pub fn with_asset_workers(mut self, n: usize) -> Self {
        self.asset_workers = n;
        self
    }

    pub fn with_asset_root<P>(mut self, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.asset_root = Some(path.into());
        self
    }

    pub fn build(mut self) -> App {
        let mut app = App::new(self.asset_workers, self.asset_root);

        for b in mem::take(&mut self.windows) {
            app.queued_windows.push(b);
        }

        app
    }
}
