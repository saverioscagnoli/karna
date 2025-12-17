use crate::{App, scene::Scene};
use macros::With;
use math::Size;
use utils::{
    label,
    map::{Label, LabelMap},
};
use winit::window::WindowAttributes;

#[derive(Default)]
#[derive(With)]
pub struct WindowBuilder {
    pub(crate) attributes: WindowAttributes,

    #[with(into)]
    /// Used for debugging purposes,
    /// such as distinguish logs between windows
    pub(crate) label: String,
    pub(crate) scenes: LabelMap<Box<dyn Scene>>,
}

impl WindowBuilder {
    /// Creates a new `WindowBuilder`
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the window title before creation
    pub fn with_title<T: Into<String>>(mut self, title: T) -> Self {
        self.attributes = self.attributes.with_title(title);
        self
    }

    /// Sets the size of the window before creation
    pub fn with_size<S: Into<Size<u32>>>(mut self, size: S) -> Self {
        let size: Size<u32> = size.into();

        self.attributes = self.attributes.with_inner_size(size);
        self
    }

    /// Sets whether the window should be resizable before creation.
    ///
    /// NOTE: on tiling window managers, setting this to `true`
    /// will spawn the window as floating, overriding the tiling rules
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.attributes = self.attributes.with_resizable(resizable);
        self
    }

    /// Adds a scene to the window
    ///
    /// NOTE: To add an intial scene when the window spawns,
    /// use `with_initial_scene`, which is mandatory to do
    /// for each window
    pub fn with_scene<S: Scene + 'static>(mut self, label: Label, scene: S) -> Self {
        self.scenes.insert(label, Box::new(scene));
        self
    }

    /// Sets the initial scene of the window
    pub fn with_initial_scene<S: Scene + 'static>(mut self, scene: S) -> Self {
        self.scenes.insert(label!("initial"), Box::new(scene));
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

    /// Creates a new window
    pub fn with_window(mut self, window: WindowBuilder) -> Self {
        self.windows.push(window);
        self
    }

    /// Creates a new app
    pub fn build(self) -> App {
        let mut app = App::new();

        for (i, mut builder) in self.windows.into_iter().enumerate() {
            assert!(
                builder.scenes.contains_key(&label!("initial")),
                "WindowBuilder must have an initial scene. Use with_initial_scene() or with_scene(label!(\"initial\"), scene)"
            );

            builder.label = if builder.label.is_empty() {
                format!("window {}", i + 1)
            } else {
                builder.label
            };

            app.add_window_builder(builder);
        }

        app
    }
}
