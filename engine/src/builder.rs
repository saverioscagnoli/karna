use std::collections::HashMap;

use math::Size;
use winit::dpi::PhysicalSize;
use winit::window::WindowAttributes;

use crate::App;
use crate::scene::Scene;

#[derive(Default)]
pub struct WindowBuilder {
    attrs: WindowAttributes,
    scenes: HashMap<String, Box<dyn Scene>>,
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

    pub fn with_scene<S: Scene + 'static>(mut self, label: &str, scene: S) -> Self {
        self.scenes.insert(label.to_owned(), Box::new(scene));
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

        for (i, b) = self.windows.iter().enumerate() {}
    }
}
