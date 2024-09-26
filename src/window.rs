use crate::{math::Size, throw, Error};

use std::sync::OnceLock;

use sdl2::video::Window;

static mut WINDOW: OnceLock<Window> = OnceLock::new();

pub fn init(window: Window) {
    unsafe {
        WINDOW
            .set(window)
            .map_err(|_| Error::Window("Failed to initialize window".to_string()))
            .unwrap();
    }
}

pub(crate) fn window() -> &'static Window {
    unsafe {
        match WINDOW.get() {
            Some(window) => window,
            None => throw!(Error::Window(
                "Window not initialized! Did you forget to call `crate_window`?".to_string(),
            )),
        }
    }
}

fn window_mut() -> &'static mut Window {
    unsafe {
        match WINDOW.get_mut() {
            Some(window) => window,
            None => throw!(Error::Window(
                "Window not initialized! Did you forget to call `crate_window`?".to_string(),
            )),
        }
    }
}

pub fn title() -> String {
    window().title().to_string()
}

pub fn set_title(title: impl ToString) -> Result<(), Error> {
    window_mut()
        .set_title(&title.to_string())
        .map_err(|_| Error::Window("Failed to set title.".to_string()))
}

pub fn size() -> Size {
    window().size().into()
}

pub fn set_size(size: Size) -> Result<(), Error> {
    window_mut()
        .set_size(size.width, size.height)
        .map_err(|_| Error::Window("Failed to set size.".to_string()))
}

pub fn position() -> Size {
    window().position().into()
}
