use fontdue::layout::{CoordinateSystem, Layout};
use sdl2::render::Texture;
use std::{collections::HashMap, ops::Deref, rc::Rc};

pub(crate) struct Font {
    // Need to make it an rc because when we pass it to the layout,
    // it needs a slice of the fonts, and cant clone it every time
    pub(crate) inner: Rc<fontdue::Font>,
    pub(crate) size: f32,
    pub(crate) char_cache: HashMap<char, Texture<'static>>,
    pub(crate) layout: Layout,
}

impl Font {
    pub fn new(fontdue_font: fontdue::Font, size: f32) -> Self {
        Self {
            inner: Rc::new(fontdue_font),
            size,
            char_cache: HashMap::new(),
            layout: Layout::new(CoordinateSystem::PositiveYDown),
        }
    }
}

impl Deref for Font {
    type Target = fontdue::Font;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
