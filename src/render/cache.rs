use std::collections::HashMap;

use sdl2::{pixels::Color, render::Texture};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum TextureType {
    CircleOutline(u32, Color),
    CircleFill(u32, Color),
    Text(String, Color),
}

impl TextureType {
    pub fn key(&self) -> String {
        match self {
            Self::Text(s, c) => format!("{s}-{c:?}"),
            Self::CircleFill(r, c) => format!("c-f-{r}-{c:?}"),
            Self::CircleOutline(r, c) => format!("c-o-{r}-{c:?}"),
        }
    }
}

pub(crate) struct TextureCache {
    cache: HashMap<String, Texture>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get_or_insert<F>(&mut self, texture_type: TextureType, f: F) -> &Texture
    where
        F: FnOnce() -> Texture,
    {
        let key = texture_type.key();
        self.cache.entry(key).or_insert_with(f)
    }
}
