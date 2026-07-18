use crate::App;
use crate::scene::Scene;
use crate::scene::SceneRegistry;

use sdl3::video::Window as SdlWindow;
use sdl3::video::WindowBuilder as SdlWindowBuilder;

#[derive(Default)]
pub struct WindowBuilder {
    pub(crate) title: String,
    pub(crate) size: math::Size<u32>,
    pub(crate) resizable: bool,
    pub(crate) scenes: SceneRegistry,
    pub(crate) initial_active: Vec<usize>,
}

impl WindowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        let title: String = title.into();
        self.title = title;
        self
    }

    pub fn with_size<S>(mut self, size: S) -> Self
    where
        S: Into<math::Size<u32>>,
    {
        let size: math::Size<u32> = size.into();
        self.size = size;
        self
    }

    pub fn with_resizable(mut self, r: bool) -> Self {
        self.resizable = r;
        self
    }

    pub fn with_scene<S>(mut self, index: usize, scene: S) -> Self
    where
        S: Scene + 'static,
    {
        self.scenes.insert(index, scene);
        self
    }

    pub fn with_active_scene(mut self, index: usize) -> Self {
        self.initial_active.push(index);
        self
    }

    pub(crate) fn build_sdl(&self, mut sdl: SdlWindowBuilder) -> SdlWindow {
        if self.resizable {
            sdl.resizable();
        }

        sdl.build().expect("Failed to build window")
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

    pub fn build(self) -> App {
        let mut app = App::new();

        for (_, b) in self.windows.into_iter().enumerate() {
            app.queue_window(b);
        }

        return app;
    }
}
