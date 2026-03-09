use std::sync::atomic::AtomicU32;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use macros::Get;
use macros::With;
use math::Size;
use utils::FastHashMap;
use winit::window::WindowAttributes;

use crate::App;
use crate::scene::Scene;
use crate::scene::SceneMap;

static ID_AUTOINC: AtomicUsize = AtomicUsize::new(1);

fn window_id() -> usize {
    ID_AUTOINC.fetch_add(1, Ordering::Relaxed)
}

#[derive(Get, With)]
pub struct WindowBuilder {
    #[get(copied)]
    id: usize,

    pub(crate) scenes: SceneMap,

    #[with(into)]
    size: Size<u32>,

    #[with(into)]
    title: String,

    #[with]
    resizable: bool,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            id: window_id(),
            scenes: FastHashMap::default(),
            size: Size::new(800, 600),
            title: String::from("My Window"),
            resizable: true,
        }
    }
}

impl WindowBuilder {
    pub fn new() -> Self {
        WindowBuilder::default()
    }

    pub fn with_scene<S: Scene + 'static>(mut self, label: impl Into<String>, scene: S) -> Self {
        self.scenes.insert(label.into(), Box::new(scene));
        self
    }

    pub fn with_initial_scene<S: Scene + 'static>(mut self, scene: S) -> Self {
        self.scenes.insert(String::from("initial"), Box::new(scene));
        self
    }

    pub(crate) fn attributes(&self) -> WindowAttributes {
        WindowAttributes::default()
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.size.width,
                self.size.height,
            ))
            .with_title(self.title.clone())
            .with_resizable(self.resizable)
    }
}

pub struct AppBuilder {
    windows: Vec<WindowBuilder>,
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
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

    pub fn build(self) -> App {
        let mut app = App::new();

        for b in self.windows {
            app.queue_window(b);
        }

        app
    }
}
