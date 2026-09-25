use nostd::alloc::boxed::Box;
use nostd::alloc::string::String;
use nostd::alloc::vec::Vec;
use nostd::collections::HashMap;
use nostd::path::Path;
use nostd::path::PathBuf;
use traccia::debug;

use crate::App;
use crate::context::LoadContext;
use crate::scene::BoxedScene;
use crate::scene::Scene;
use crate::scene::SceneBuilder;
use crate::scene::SceneId;

pub struct WindowBuilder {
    pub title: String,
    pub size: math::Size<u32>,
    pub resizable: bool,
    pub decorated: bool,
    pub always_on_top: bool,
    pub transparent: bool,
    pub opacity: f32,
    pub focusable: bool,
    pub high_pixel_density: bool,
    pub grab_mouse: bool,
    pub grab_keyboard: bool,
    pub scene_builders: HashMap<SceneId, SceneBuilder>,
    pub active_scenes: Vec<SceneId>,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: String::from("My Window"),
            size: math::size!(800, 600),
            resizable: false,
            decorated: true,
            always_on_top: false,
            transparent: false,
            opacity: 1.0,
            focusable: true,
            high_pixel_density: false,
            grab_mouse: false,
            grab_keyboard: false,
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

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    pub fn with_decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    pub fn with_always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }

    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    pub fn with_opacity(mut self, value: f32) -> Self {
        self.opacity = value;
        self
    }

    pub fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    pub fn with_high_pixel_density(mut self, enabled: bool) -> Self {
        self.high_pixel_density = enabled;
        self
    }

    pub fn with_grab_mouse(mut self, grab_mouse: bool) -> Self {
        self.grab_mouse = grab_mouse;
        self
    }

    pub fn with_grab_keyboard(mut self, grab_keyboard: bool) -> Self {
        self.grab_keyboard = grab_keyboard;
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

    pub fn with_scene_fn<F>(mut self, id: SceneId, f: F) -> Self
    where
        F: Fn(&mut LoadContext) -> BoxedScene + 'static,
    {
        self.scene_builders.insert(id, Box::new(f));
        self
    }

    pub fn with_active_scene(mut self, id: SceneId) -> Self {
        self.active_scenes.push(id);
        self
    }
}

pub struct AppBuilder {
    windows: Vec<WindowBuilder>,
    root: PathBuf,
    workers: usize,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
            root: Path::new(".").to_path_buf(),
            workers: 4,
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

    pub fn with_workers(mut self, workers: usize) -> Self {
        self.workers = workers;
        self
    }

    pub fn build(self) -> App {
        debug!("Requested creation of {} window(s)", self.windows.len());
        debug!("Resolved root path: {}", self.root);

        let mut app = App::new(self.root, self.workers);

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
