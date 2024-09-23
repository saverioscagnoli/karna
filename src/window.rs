use sdl2::video::Window;
use std::cell::OnceCell;

use crate::math::Size;

pub(crate) static mut WINDOW: OnceCell<Window> = OnceCell::new();

pub fn title() -> String {
    unsafe { WINDOW.get().unwrap().title().to_string() }
}

pub fn set_title(title: impl ToString) {
    unsafe {
        WINDOW
            .get_mut()
            .unwrap()
            .set_title(&title.to_string())
            .unwrap();
    }
}

pub fn size() -> Size {
    unsafe { WINDOW.get().unwrap().size() }.into()
}

pub fn set_size(size: impl Into<Size>) {
    let size = size.into();
    let (width, height) = (size.width as u32, size.height as u32);

    unsafe {
        WINDOW.get_mut().unwrap().set_size(width, height).unwrap();
    }
}
