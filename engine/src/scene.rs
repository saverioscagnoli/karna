use std::hash::Hash;

use crate::render::draw::Draw;
use crate::render::scene_ref::SceneRef;
use crate::window::context::DrawContext;
use crate::window::context::LoadContext;
use crate::window::context::UpdateContext;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneId(u64);

pub const fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    hash
}

impl SceneId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn new_label(label: &'static str) -> Self {
        Self(fnv1a(label.as_bytes()))
    }
}

#[allow(unused)]
pub trait Scene: 'static {
    fn load(ctx: LoadContext, scene: &mut SceneRef) -> Self
    where
        Self: Sized;

    fn fixed_update(&mut self, ctx: UpdateContext, scene: &mut SceneRef) {}
    fn update(&mut self, ctx: UpdateContext, scene: &mut SceneRef);
    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw);
}

pub type SceneBuilder = Box<dyn FnOnce(LoadContext, &mut SceneRef) -> Box<dyn Scene>>;
