use std::rc::Rc;

use lfu_cache::LfuCache;
use sdl2::{pixels::Color, render::Texture};

#[derive(Debug)]
pub(crate) enum TextureKind {
    Char(char, Color),
    Text(Rc<str>, Color),
    CircleOutline(u32, Color),
    CircleFilled(u32, Color),
    CircleFilledAA(u32, Color),
}

impl TextureKind {
    pub(crate) fn key(&self) -> String {
        match self {
            TextureKind::Char(ch, color) => format!("char-{}-{:?}", ch, color),
            TextureKind::Text(text, color) => format!("text-{}-{:?}", text, color),
            TextureKind::CircleOutline(radius, color) => format!("c-o-{}-{:?}", radius, color),
            TextureKind::CircleFilled(radius, color) => format!("c-f-{}-{:?}", radius, color),
            TextureKind::CircleFilledAA(radius, color) => format!("c-f-aa-{}-{:?}", radius, color),
        }
    }
}

pub(crate) struct TextureCache {
    pub(crate) chars: LfuCache<Rc<str>, Texture<'static>>,
    pub(crate) text: LfuCache<Rc<str>, Texture<'static>>,
    pub(crate) shapes: LfuCache<Rc<str>, Texture<'static>>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            chars: LfuCache::with_capacity(20_000),
            text: LfuCache::with_capacity(20_000),
            shapes: LfuCache::with_capacity(20_000),
        }
    }

    pub fn get(&mut self, kind: &TextureKind) -> Option<&Texture<'static>> {
        match kind {
            TextureKind::Char(_, _) => self.chars.get(&kind.key().into()),
            TextureKind::Text(_, _) => self.text.get(&kind.key().into()),
            TextureKind::CircleOutline(_, _) => self.shapes.get(&kind.key().into()),
            TextureKind::CircleFilled(_, _) => self.shapes.get(&kind.key().into()),
            TextureKind::CircleFilledAA(_, _) => self.shapes.get(&kind.key().into()),
        }
    }

    pub fn insert(&mut self, kind: &TextureKind, texture: Texture<'static>) {
        match kind {
            TextureKind::Char(_, _) => {
                self.chars.insert(kind.key().into(), texture);
            }
            TextureKind::Text(_, _) => {
                self.text.insert(kind.key().into(), texture);
            }
            TextureKind::CircleOutline(_, _) => {
                self.shapes.insert(kind.key().into(), texture);
            }
            TextureKind::CircleFilled(_, _) => {
                self.shapes.insert(kind.key().into(), texture);
            }
            TextureKind::CircleFilledAA(_, _) => {
                self.shapes.insert(kind.key().into(), texture);
            }
        }
    }
}
