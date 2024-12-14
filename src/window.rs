use std::path::Path;

use crate::{
    math::{Size, ToF32, Vec2},
    traits::LoadSurface,
};
use sdl2::{surface::Surface, video::WindowPos};

pub struct Window(sdl2::video::Window);

impl Window {
    pub(crate) fn new(sdl_win: sdl2::video::Window) -> Self {
        Self(sdl_win)
    }

    pub fn hide(&mut self) {
        self.0.hide();
    }

    pub fn show(&mut self) {
        self.0.show();
    }

    pub fn size(&self) -> Size {
        self.0.size().into()
    }

    pub fn set_size<S: Into<Size>>(&mut self, size: S) {
        let size: Size = size.into();
        self.0.set_size(size.width, size.height).unwrap();
    }

    pub fn title(&self) -> String {
        self.0.title().to_string()
    }

    pub fn set_title<T: ToString>(&mut self, title: T) {
        self.0.set_title(title.to_string().as_str()).unwrap();
    }

    pub fn set_resizable(&mut self, val: bool) {
        self.0.set_resizable(val);
    }

    pub fn position(&self) -> Vec2 {
        self.0.position().into()
    }

    pub fn center_position(&self) -> Vec2 {
        let size = self.size();
        Vec2::new(size.width.to_f32() / 2.0, size.height.to_f32() / 2.0)
    }

    pub fn set_position<P: Into<Vec2>>(&mut self, pos: P) {
        let pos: Vec2 = pos.into();
        self.0.set_position(
            WindowPos::Positioned(pos.x as i32),
            WindowPos::Positioned(pos.y as i32),
        );
    }

    pub fn center(&mut self) {
        self.0
            .set_position(WindowPos::Centered, WindowPos::Centered);
    }

    pub fn fullscreen(&mut self) {
        self.0
            .set_fullscreen(sdl2::video::FullscreenType::Desktop)
            .unwrap();
    }

    pub fn windowed(&mut self) {
        self.0
            .set_fullscreen(sdl2::video::FullscreenType::Off)
            .unwrap();
    }

    pub fn set_decorations(&mut self, val: bool) {
        self.0.set_bordered(val);
    }

    pub fn set_icon<P: AsRef<Path>>(&mut self, path: P) {
        let surface = Surface::from_file(path);
        self.0.set_icon(surface)
    }
}
