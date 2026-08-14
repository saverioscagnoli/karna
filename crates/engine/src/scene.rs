use std::hash::Hash;

use crate::render::draw::Draw;
use crate::render::stage::SceneView;
use crate::window::context::DrawContext;
use crate::window::context::LoadContext;
use crate::window::context::UpdateContext;

#[derive(Default)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SceneId(u64);

impl SceneId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn new_label(label: &'static str) -> Self {
        Self(utils::fnv1a(label.as_bytes()))
    }
}

#[allow(unused)]
pub trait Scene: 'static {
    fn load(ctx: LoadContext, scene: &mut SceneView) -> Self
    where
        Self: Sized;

    fn fixed_update(&mut self, ctx: UpdateContext, scene: &mut SceneView) {}
    fn update(&mut self, ctx: UpdateContext, scene: &mut SceneView);
    fn draw(&mut self, ctx: DrawContext, draw: &mut Draw);
}

pub type SceneBuilder = Box<dyn FnOnce(LoadContext, &mut SceneView) -> Box<dyn Scene>>;
pub type BoxedScene = Box<dyn Scene>;
