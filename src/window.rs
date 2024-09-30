use crate::{math::Size, throw, Error};

use std::{cell::OnceCell, collections::HashMap, sync::OnceLock};

pub use sdl2::mouse::SystemCursor as Cursor;
use sdl2::{image::LoadSurface, mouse::Cursor as SdlCursor, surface::Surface, video::Window};

static mut WINDOW: OnceLock<Window> = OnceLock::new();
static mut CURSORS: OnceCell<HashMap<String, SdlCursor>> = OnceCell::new();

pub fn init(window: Window) {
    unsafe {
        WINDOW
            .set(window)
            .map_err(|_| Error::Window("Failed to initialize window".to_string()))
            .unwrap();

        CURSORS
            .set(HashMap::new())
            .map_err(|_| Error::Window("Failed to initialize cursors".to_string()))
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

fn cursors_mut() -> &'static mut HashMap<String, SdlCursor> {
    unsafe { CURSORS.get_mut().unwrap() }
}

pub fn load_cursor<L, P>(label: L, path: P)
where
    L: Into<String>,
    P: Into<String>,
{
    let label = label.into();
    let path = path.into();
    let surface = Surface::from_file(&path).unwrap();
    let cursor = SdlCursor::from_surface(surface, 0, 0).unwrap();

    cursors_mut().insert(label, cursor);
}

pub fn load_system_cursor<L>(label: L, cursor: Cursor)
where
    L: Into<String>,
{
    let label = label.into();
    let cursor = SdlCursor::from_system(cursor).unwrap();
    cursors_mut().insert(label, cursor);
}

pub fn set_cursor(label: impl ToString) {
    let label = label.to_string();
    let cursor = cursors_mut().get(&label).unwrap();
    cursor.set();
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
