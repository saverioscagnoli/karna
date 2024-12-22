use std::path::Path;

use crate::{
    math::{Size, ToF32, Vec2},
    traits::LoadSurface,
};
use hashbrown::HashMap;
use sdl2::{mouse::SystemCursor, surface::Surface, video::WindowPos};

pub enum Cursor {
    Arrow,
    IBeam,
    Wait,
    Crosshair,
    WaitArrow,
    SizeNWSE,
    SizeNESW,
    SizeWE,
    SizeNS,
    SizeAll,
    No,
    Hand,
    Custom(&'static str),
}

impl ToString for Cursor {
    fn to_string(&self) -> String {
        match self {
            Cursor::Arrow => "Arrow".to_string(),
            Cursor::IBeam => "IBeam".to_string(),
            Cursor::Wait => "Wait".to_string(),
            Cursor::Crosshair => "Crosshair".to_string(),
            Cursor::WaitArrow => "WaitArrow".to_string(),
            Cursor::SizeNWSE => "SizeNWSE".to_string(),
            Cursor::SizeNESW => "SizeNESW".to_string(),
            Cursor::SizeWE => "SizeWE".to_string(),
            Cursor::SizeNS => "SizeNS".to_string(),
            Cursor::SizeAll => "SizeAll".to_string(),
            Cursor::No => "No".to_string(),
            Cursor::Hand => "Hand".to_string(),
            Cursor::Custom(label) => label.to_string(),
        }
    }
}

pub struct Window {
    pub(crate) inner: sdl2::video::Window,
    cursors: HashMap<String, sdl2::mouse::Cursor>,
}

impl Window {
    pub(crate) fn new(sdl_win: sdl2::video::Window) -> Self {
        Self {
            inner: sdl_win,
            cursors: Self::setup_cursors(),
        }
    }

    fn setup_cursors() -> HashMap<String, sdl2::mouse::Cursor> {
        let system_cursors = vec![
            ("Arrow", SystemCursor::Arrow),
            ("IBeam", SystemCursor::IBeam),
            ("Wait", SystemCursor::Wait),
            ("Crosshair", SystemCursor::Crosshair),
            ("WaitArrow", SystemCursor::WaitArrow),
            ("SizeNWSE", SystemCursor::SizeNWSE),
            ("SizeNESW", SystemCursor::SizeNESW),
            ("SizeWE", SystemCursor::SizeWE),
            ("SizeNS", SystemCursor::SizeNS),
            ("SizeAll", SystemCursor::SizeAll),
            ("No", SystemCursor::No),
            ("Hand", SystemCursor::Hand),
        ];

        let mut cursors = HashMap::new();

        for (label, cursor) in system_cursors {
            let sdl_cursor = sdl2::mouse::Cursor::from_system(cursor).unwrap();
            cursors.insert(label.to_string(), sdl_cursor);
        }

        cursors
    }

    pub(crate) fn swap_buffers(&self) {
        self.inner.gl_swap_window();
    }

    pub fn hide(&mut self) {
        self.inner.hide();
    }

    pub fn show(&mut self) {
        self.inner.show();
    }

    pub fn size(&self) -> Size {
        self.inner.size().into()
    }

    pub fn set_size<S: Into<Size>>(&mut self, size: S) {
        let size: Size = size.into();
        self.inner.set_size(size.width, size.height).unwrap();
    }

    pub fn title(&self) -> String {
        self.inner.title().to_string()
    }

    pub fn set_title<T: ToString>(&mut self, title: T) {
        self.inner.set_title(title.to_string().as_str()).unwrap();
    }

    pub fn set_resizable(&mut self, val: bool) {
        self.inner.set_resizable(val);
    }

    pub fn position(&self) -> Vec2 {
        self.inner.position().into()
    }

    pub fn center_position(&self) -> Vec2 {
        let size = self.size();
        Vec2::new(size.width.to_f32() / 2.0, size.height.to_f32() / 2.0)
    }

    pub fn set_position<P: Into<Vec2>>(&mut self, pos: P) {
        let pos: Vec2 = pos.into();
        self.inner.set_position(
            WindowPos::Positioned(pos.x as i32),
            WindowPos::Positioned(pos.y as i32),
        );
    }

    pub fn center(&mut self) {
        self.inner
            .set_position(WindowPos::Centered, WindowPos::Centered);
    }

    pub fn fullscreen(&mut self) {
        self.inner
            .set_fullscreen(sdl2::video::FullscreenType::Desktop)
            .unwrap();
    }

    pub fn windowed(&mut self) {
        self.inner
            .set_fullscreen(sdl2::video::FullscreenType::Off)
            .unwrap();
    }

    pub fn set_decorations(&mut self, val: bool) {
        self.inner.set_bordered(val);
    }

    pub fn set_icon<P: AsRef<Path>>(&mut self, path: P) {
        let surface = Surface::from_file(path);
        self.inner.set_icon(surface)
    }

    pub fn load_cursor<L: ToString, P: AsRef<Path>>(&mut self, label: L, path: P) {
        let surface = Surface::from_file(path);
        let cursor = sdl2::mouse::Cursor::from_surface(surface, 0, 0).unwrap();
        self.cursors.insert(label.to_string(), cursor);
    }

    pub fn set_cursor(&self, cursor: Cursor) {
        if let Some(cursor) = self.cursors.get(&cursor.to_string()) {
            cursor.set();
        }
    }
}
