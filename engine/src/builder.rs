use utils::FastHashMap;

use crate::App;
use crate::scene::Scene;
use crate::scene::SceneBuilder;
use crate::scene::SceneId;

pub struct WindowBuilder {
    pub title: String,
    pub size: math::Size<u32>,
    pub scenes: FastHashMap<SceneId, SceneBuilder>,
    pub active_scenes: Vec<SceneId>,
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

    pub fn with_scene<S>(mut self, id: SceneId) -> Self
    where
        S: Scene,
    {
        self.scenes
            .insert(id, Box::new(|ctx, scene| Box::new(S::load(ctx, scene))));
        self
    }

    pub fn with_active_scene(mut self, id: SceneId) -> Self {
        self.active_scenes.push(id);
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

        for w in self.windows {
            app.requested_at_creation.push(w);
        }

        app
    }
}
