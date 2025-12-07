use crate::{App, Scene, init_logging};
use common::{label, utils::Label};
use math::Size;
use wgpu::naga::FastHashMap;
use winit::window::WindowAttributes;

pub struct WindowBuilder {
    attributes: WindowAttributes,
    scenes: FastHashMap<Label, Box<dyn Scene>>,
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            attributes: WindowAttributes::default()
                .with_title("My Window")
                .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
            scenes: FastHashMap::default(),
        }
    }
}

impl WindowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.attributes = self.attributes.with_title(title);
        self
    }

    pub fn with_size<S: Into<Size<u32>>>(mut self, size: S) -> Self {
        let size = size.into();
        self.attributes = self
            .attributes
            .with_inner_size(winit::dpi::LogicalSize::new(size.width, size.height));
        self
    }

    pub fn with_initial_scene(mut self, scene: Box<dyn Scene>) -> Self {
        self.scenes.insert(label!("initial"), scene);
        self
    }

    pub fn with_scene(mut self, label: Label, scene: Box<dyn Scene>) -> Self {
        self.scenes.insert(label, scene);
        self
    }

    pub(crate) fn build(self) -> (WindowAttributes, FastHashMap<Label, Box<dyn Scene>>) {
        assert!(
            self.scenes.contains_key(&label!("initial")),
            "WindowBuilder must have an initial scene. Use with_initial_scene() or with_scene(label!(\"initial\"), scene)"
        );

        (self.attributes.with_resizable(false), self.scenes)
    }
}

pub struct AppBuilder {
    windows: Vec<WindowBuilder>,
}

impl AppBuilder {
    pub fn new() -> Self {
        init_logging();
        Self {
            windows: Vec::new(),
        }
    }

    pub fn with_window(mut self, window: WindowBuilder) -> Self {
        self.windows.push(window);
        self
    }

    pub fn build(self) -> App {
        let mut app = App::new();

        // Add all windows as pending - they'll be created in resumed()
        for window_builder in self.windows {
            let (attributes, scenes) = window_builder.build();
            app.add_pending_window(attributes, scenes);
        }

        app
    }
}
