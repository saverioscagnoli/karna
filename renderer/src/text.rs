use crate::{Color, Geometry, Material, Mesh, Transform};
use assets::Font;
use macros::{Get, Set, With};
use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use utils::{Lazy, map::Label};

#[derive(Debug)]
#[derive(Get, Set, With)]
pub struct Text {
    #[get(visibility = "pub(crate)")]
    font_label: Label,

    #[get(ty = &str)]
    #[get(mut, also = self.mark())]
    #[set(into, also = self.mark())]
    #[with(into)]
    content: String,
    dirty: Cell<bool>,
}

impl Text {
    pub fn new(font_label: Label) -> Self {
        Self {
            font_label,
            content: String::new(),
            dirty: Cell::new(false),
        }
    }

    #[inline]
    fn mark(&self) {
        self.dirty.set(true);
    }
}
