use sokol::app as sapp;

pub struct Window {
    title: String,
    size: math::Size<u32>,
}

impl Window {
    pub(crate) fn new<T: Into<String>>(title: T, size: math::Size<u32>) -> Self {
        Self {
            title: title.into(),
            size,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title<T: Into<String>>(&mut self, title: T) {
        self.title = title.into();
        sapp::set_window_title(&self.title);
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn size(&self) -> math::Size<u32> {
        self.size
    }

    pub fn is_fullscreen(&self) -> bool {
        sapp::is_fullscreen()
    }

    pub fn toggle_fullscreen(&self) {
        sapp::toggle_fullscreen();
    }

    pub fn request_quit(&self) {
        sapp::request_quit();
    }
}
