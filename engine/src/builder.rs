use math::Size;
use winit::dpi::PhysicalSize;
use winit::window::WindowAttributes;

use crate::App;
use crate::ContextRefMut;
use crate::scene::Scene;
use crate::scene::Scenes;

#[derive(Default)]
pub struct WindowBuilder {
    pub(crate) attrs: WindowAttributes,
    pub(crate) scenes: Scenes,
    pub(crate) initial_active: Vec<String>,
}

impl WindowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        let title: String = title.into();
        self.attrs = self.attrs.with_title(title);
        self
    }

    pub fn with_size<S: Into<Size<u32>>>(mut self, size: S) -> Self {
        let size: Size<u32> = size.into();
        let size = PhysicalSize::new(size.width, size.height);
        self.attrs = self.attrs.with_inner_size(size);
        self
    }

    pub fn with_resizable(mut self, r: bool) -> Self {
        self.attrs = self.attrs.with_resizable(r);
        self
    }

    pub fn with_scene<S: Scene + 'static>(self, label: impl Into<String>, scene: S) -> Self {
        self.build_scene(label, move |_ctx| scene)
    }

    pub fn build_scene<S, F, L: Into<String>>(mut self, label: L, f: F) -> Self
    where
        S: Scene + 'static,
        F: FnOnce(ContextRefMut) -> S + Send + 'static,
    {
        self.scenes
            .insert_builder(label.into(), Box::new(move |ctx| Box::new(f(ctx))));

        self
    }

    pub fn with_active_scene(mut self, label: &str) -> Self {
        self.initial_active.push(label.to_owned());
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

    pub fn build(self) -> App {
        let mut app = App::new();

        for (_, b) in self.windows.into_iter().enumerate() {
            app.request_window(b);
        }

        return app;
    }
}
